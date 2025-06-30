use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[cfg(feature = "browser")]
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey, ModifiersState},
    window::{Window, WindowId},
};
#[cfg(feature = "browser")]
use wry::{PageLoadEvent, WebView, WebViewBuilder};

pub struct History {
    entries: RefCell<Vec<String>>,
    index: Cell<usize>,
}

impl History {
    pub fn new(initial: String) -> Self {
        Self {
            entries: RefCell::new(vec![initial]),
            index: Cell::new(0),
        }
    }

    pub fn push(&self, url: String) {
        let mut entries = self.entries.borrow_mut();
        let idx = self.index.get();
        if entries.get(idx).map_or(false, |u| u == &url) {
            return;
        }
        entries.truncate(idx + 1);
        entries.push(url);
        self.index.set(entries.len() - 1);
    }

    pub fn current(&self) -> Option<String> {
        self.entries
            .borrow()
            .get(self.index.get())
            .cloned()
    }

    pub fn back(&self) -> Option<String> {
        if self.index.get() > 0 {
            self.index.set(self.index.get() - 1);
            return self.current();
        }
        None
    }

    pub fn forward(&self) -> Option<String> {
        if self.index.get() + 1 < self.entries.borrow().len() {
            self.index.set(self.index.get() + 1);
            return self.current();
        }
        None
    }
}

pub struct Browser {
    #[cfg(feature = "browser")]
    pub window: Option<Window>,
    #[cfg(feature = "browser")]
    pub webview: Option<WebView>,
    pub history: Rc<History>,
    #[cfg(feature = "browser")]
    pub modifiers: winit::keyboard::ModifiersState,
}

#[cfg(feature = "browser")]
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

#[cfg(feature = "browser")]
pub fn run(initial_url: String) -> wry::Result<()> {
    let event_loop = EventLoop::new().unwrap();
    let mut browser = Browser {
        window: None,
        webview: None,
        history: Rc::new(History::new(initial_url)),
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
