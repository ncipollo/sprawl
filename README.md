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

### Server and client

Sections can run on another machine:

    cargo run -- serve --interval 300 --port 7878     # headless; renders to ~/.sprawl/output and serves HTTP
    cargo run -- --connect http://host:7878           # the app, fed by that server

The server listens on 127.0.0.1 by default and has no authentication; pass
`--host` to expose it, only on networks you trust. Endpoints: `GET /sections`,
`GET /sections/{id}`, `POST /sections/{id}/refresh`.

## Develop

    cargo fmt
    cargo test
    cargo clippy
