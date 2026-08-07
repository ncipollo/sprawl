# sprawl

What was I doing? Where did it go?

A desktop app built with [gpui](https://www.gpui.rs/), Zed's GPU-accelerated UI framework.

## Requirements

- Stable Rust (edition 2024)
- macOS with Xcode installed and selected (`sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer`) — gpui renders with Metal

## Run

    cargo run

Opens a 1024×720 window, centered on screen, with a navigation sidebar on the
left and a content pane on the right.

## Develop

    cargo fmt
    cargo test
    cargo clippy
