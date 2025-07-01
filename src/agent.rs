// Browser agent implementations
use crate::Browser;

/// Trait for agents controlling the [`Browser`].
pub trait BrowserAgent {
    /// Returns the next command, if any.
    fn next_command(&mut self) -> Option<String>;

    /// Processes a command and applies it to the browser.
    fn process_command(&self, browser: &mut Browser, cmd: &str) {
        let cmd = cmd.trim();
        if cmd == "back" {
            if let Some(_url) = browser.history.back() {
                #[cfg(feature = "browser")]
                if let Some(wv) = &browser.webview {
                    wv.load_url(&_url).ok();
                }
            }
        } else if cmd == "forward" {
            if let Some(_url) = browser.history.forward() {
                #[cfg(feature = "browser")]
                if let Some(wv) = &browser.webview {
                    wv.load_url(&_url).ok();
                }
            }
        } else if let Some(rest) = cmd.strip_prefix("go ") {
            #[cfg(feature = "browser")]
            if let Some(wv) = &browser.webview {
                wv.load_url(rest).ok();
            }
            browser.history.push(rest.to_string());
        }
    }
}

/// Agent that reads commands from standard input.
pub struct StdinAgent;

impl StdinAgent {
    pub fn new() -> Self {
        Self
    }
}

impl BrowserAgent for StdinAgent {
    fn next_command(&mut self) -> Option<String> {
        use std::io::{self, Write};
        print!("command> ");
        io::stdout().flush().ok()?;
        let mut buf = String::new();
        if io::stdin().read_line(&mut buf).ok()? == 0 {
            return None;
        }
        Some(buf.trim().to_string())
    }
}

#[cfg(feature = "ai")]
use async_openai::Client;

/// Agent backed by OpenAI. Currently returns no commands until implemented.
#[cfg(feature = "ai")]
pub struct OpenAIAgent {
    _client: Client,
}

#[cfg(feature = "ai")]
impl OpenAIAgent {
    pub fn new() -> Self {
        Self { _client: Client::new() }
    }
}

#[cfg(feature = "ai")]
impl BrowserAgent for OpenAIAgent {
    fn next_command(&mut self) -> Option<String> {
        // Real interaction with the LLM would go here.
        None
    }
}
