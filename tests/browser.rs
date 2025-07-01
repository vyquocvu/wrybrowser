use std::rc::Rc;

use wrybrowser::{Browser, History};
#[cfg(feature = "browser")]
use winit::keyboard::ModifiersState;

#[test]
fn browser_history_navigation() {
    let history = Rc::new(History::new("first".into()));
    let browser = Browser {
        #[cfg(feature = "browser")]
        window: None,
        #[cfg(feature = "browser")]
        webview: None,
        history: history.clone(),
        #[cfg(feature = "browser")]
        modifiers: ModifiersState::default(),
    };

    // simulate loading another page
    browser.history.push("second".into());
    assert_eq!(browser.history.current().as_deref(), Some("second"));

    // navigate back
    assert_eq!(browser.history.back(), Some("first".into()));
    assert_eq!(browser.history.current().as_deref(), Some("first"));

    // navigate forward
    assert_eq!(browser.history.forward(), Some("second".into()));
    assert_eq!(browser.history.current().as_deref(), Some("second"));
}
