use std::cell::{Cell, RefCell};
use std::rc::Rc;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey, ModifiersState},
    window::{Window, WindowId},
};
use wry::{PageLoadEvent, WebView, WebViewBuilder};

struct History {
    entries: RefCell<Vec<String>>,
    index: Cell<usize>,
}

impl History {
    fn new(initial: String) -> Self {
        Self {
            entries: RefCell::new(vec![initial]),
            index: Cell::new(0),
        }
    }

    fn push(&self, url: String) {
        let mut entries = self.entries.borrow_mut();
        let idx = self.index.get();
        if entries.get(idx).map_or(false, |u| u == &url) {
            return;
        }
        entries.truncate(idx + 1);
        entries.push(url);
        self.index.set(entries.len() - 1);
    }

    fn current(&self) -> Option<String> {
        self.entries
            .borrow()
            .get(self.index.get())
            .cloned()
    }

    fn back(&self) -> Option<String> {
        if self.index.get() > 0 {
            self.index.set(self.index.get() - 1);
            return self.current();
        }
        None
    }

    fn forward(&self) -> Option<String> {
        if self.index.get() + 1 < self.entries.borrow().len() {
            self.index.set(self.index.get() + 1);
            return self.current();
        }
        None
    }
}

struct Browser {
    window: Option<Window>,
    webview: Option<WebView>,
    history: Rc<History>,
    modifiers: winit::keyboard::ModifiersState,
}

impl ApplicationHandler for Browser {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        let history = self.history.clone();
        let current = history.current().unwrap_or_else(|| "about:blank".into());
        let webview = WebViewBuilder::new()
            .with_url(&current)
            .with_on_page_load_handler(move |event, url| {
                if let PageLoadEvent::Finished = event {
                    history.push(url);
                }
            })
            .build(&window)
            .unwrap();

        self.window = Some(window);
        self.webview = Some(webview);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    match event.logical_key {
                        Key::Named(NamedKey::BrowserBack)
                        | Key::Named(NamedKey::ArrowLeft)
                            if self.modifiers.alt_key() =>
                        {
                            if let Some(url) = self.history.back() {
                                if let Some(webview) = &self.webview {
                                    webview.load_url(&url).ok();
                                }
                            }
                        }
                        Key::Named(NamedKey::BrowserForward)
                        | Key::Named(NamedKey::ArrowRight)
                            if self.modifiers.alt_key() =>
                        {
                            if let Some(url) = self.history.forward() {
                                if let Some(webview) = &self.webview {
                                    webview.load_url(&url).ok();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }
            WindowEvent::CloseRequested => std::process::exit(0),
            _ => {}
        }
    }
}

fn main() -> wry::Result<()> {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://example.com".to_string());

    let event_loop = EventLoop::new().unwrap();
    let mut browser = Browser {
        window: None,
        webview: None,
        history: Rc::new(History::new(url)),
        modifiers: ModifiersState::default(),
    };
    event_loop.run_app(&mut browser).unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::History;

    #[test]
    fn history_navigation() {
        let history = History::new("a".into());
        history.push("b".into());
        history.push("c".into());

        assert_eq!(history.current().as_deref(), Some("c"));

        assert_eq!(history.back(), Some("b".into()));
        assert_eq!(history.current().as_deref(), Some("b"));
        assert_eq!(history.back(), Some("a".into()));
        assert_eq!(history.back(), None);
        assert_eq!(history.current().as_deref(), Some("a"));

        assert_eq!(history.forward(), Some("b".into()));
        assert_eq!(history.forward(), Some("c".into()));
        assert_eq!(history.forward(), None);
        assert_eq!(history.current().as_deref(), Some("c"));
    }
}
