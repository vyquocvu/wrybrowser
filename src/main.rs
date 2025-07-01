fn main() -> wry::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut ai = false;
    let mut url = None;
    while let Some(arg) = args.next() {
        if arg == "--ai" {
            ai = true;
        } else {
            url = Some(arg);
        }
    }
    let url = url.unwrap_or_else(|| "https://example.com".to_string());
    if ai {
        wrybrowser::run_with_agent(url)
    } else {
        wrybrowser::run(url)
    }
}
