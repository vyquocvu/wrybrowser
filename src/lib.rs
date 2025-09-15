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
#[cfg(feature = "browser")]
use tao::dpi::{LogicalPosition, LogicalSize};

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
    pub webview: Option<Rc<WebView>>,
    #[cfg(feature = "browser")]
    pub toolbar: Option<WebView>,
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

        let size = window.inner_size();
        let toolbar_height = 40.0;

        let content_bounds = wry::Rect {
            position: LogicalPosition::new(0.0, toolbar_height).into(),
            size: LogicalSize::new(size.width as f64, size.height as f64 - toolbar_height).into(),
        };

        let history = self.history.clone();
        let current = history.current().unwrap_or_else(|| "about:blank".into());
        let webview = Rc::new(
            WebViewBuilder::new()
                .with_url(&current)
                .with_bounds(content_bounds)
                .with_on_page_load_handler(move |event, url| {
                    if let PageLoadEvent::Finished = event {
                        history.push(url);
                    }
                })
                .build(&window)
                .unwrap(),
        );

        let content_clone = webview.clone();
        let hist = self.history.clone();
        let toolbar_bounds = wry::Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(size.width as f64, toolbar_height).into(),
        };

        const TOOLBAR_HTML: &str = r#"<input id='addr' style='width:60%'>
<button id='back'>Back</button>
<button id='forward'>Forward</button>
<script>
document.getElementById('back').addEventListener('click',()=>window.ipc.postMessage('back'));
document.getElementById('forward').addEventListener('click',()=>window.ipc.postMessage('forward'));
document.getElementById('addr').addEventListener('keydown',e=>{if(e.key==='Enter'){window.ipc.postMessage('go:'+e.target.value)}});
</script>"#;

        let toolbar = WebViewBuilder::new()
            .with_html(TOOLBAR_HTML)
            .with_bounds(toolbar_bounds)
            .with_ipc_handler(move |req| {
                let body = req.body();
                if body == "back" {
                    if let Some(url) = hist.back() {
                        content_clone.load_url(&url).ok();
                    }
                } else if body == "forward" {
                    if let Some(url) = hist.forward() {
                        content_clone.load_url(&url).ok();
                    }
                } else if let Some(rest) = body.strip_prefix("go:") {
                    content_clone.load_url(rest).ok();
                    hist.push(rest.to_string());
                }
            })
            .build(&window)
            .unwrap();

        self.window = Some(window);
        self.webview = Some(webview);
        self.toolbar = Some(toolbar);
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
pub fn run(initial_url: String) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new().unwrap();
    let mut browser = Browser {
        window: None,
        webview: None,
        toolbar: None,
        history: Rc::new(History::new(initial_url)),
        modifiers: ModifiersState::default(),
    };
    event_loop.run_app(&mut browser).unwrap();
    Ok(())
}

#[cfg(not(feature = "browser"))]
pub fn run(initial_url: String) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Headless mode: would navigate to {}", initial_url);
    eprintln!("Browser features not enabled. Build with --features browser to run the GUI.");
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
