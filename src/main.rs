fn main() -> wry::Result<()> {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://example.com".to_string());
    wrybrowser::run(url)
}
