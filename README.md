# Wry Browser

A minimal browser using the [wry](https://crates.io/crates/wry) crate.

## Build

```bash
cargo build --release
```

System packages required by WebView (such as WebKitGTK on Linux) must be installed.

## Usage

Run the binary with an optional URL argument:

```bash
cargo run -- <URL>
```

If no URL is supplied, the browser opens `https://example.com`.

Use `Alt+Left`/`Alt+Right` or dedicated browser back/forward keys to navigate
through the browsing history.

## Testing

Run the unit tests with:

```bash
cargo test
```
