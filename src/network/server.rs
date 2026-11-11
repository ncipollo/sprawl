//! The HTTP server for headless mode: list, fetch, and refresh sections.

mod handlers;
mod router;
mod state;

use crate::feature::server::service::SectionService;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

pub use router::build_router;
pub use state::AppState;

/// The default port the server listens on.
pub const DEFAULT_PORT: u16 = 7878;

/// Serves `service` on `address` until the process is stopped.
pub fn serve(service: SectionService, address: SocketAddr) -> io::Result<()> {
    let runtime = Runtime::new()?;
    runtime.block_on(async {
        let listener = TcpListener::bind(address).await?;
        eprintln!("sprawl: listening on http://{}", listener.local_addr()?);
        axum::serve(listener, build_router(AppState::new(service))).await
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::script::shell::fake::FakeShell;
    use crate::feature::server::renderer::SectionRenderer;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use http_body_util::BodyExt;
    use serde_json::Value;
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tower::ServiceExt;

    fn app(root: &TempDir) -> Router {
        let scripts = root.path().join("default");
        fs::create_dir_all(&scripts).expect("should create");
        let renderer = SectionRenderer::new(
            scripts,
            root.path().join("output"),
            Arc::new(FakeShell::outputting("")),
        );
        build_router(AppState::new(SectionService::new(renderer)))
    }

    fn write_script(root: &TempDir, name: &str, source: &str) {
        fs::create_dir_all(root.path().join("default")).expect("should create");
        fs::write(root.path().join("default").join(name), source).expect("should write");
    }

    async fn call(app: &Router, method: Method, uri: &str) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .expect("should build");
        let response = app.clone().oneshot(request).await.expect("should respond");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        (status, serde_json::from_slice(&bytes).expect("json body"))
    }

    #[tokio::test]
    async fn listing_returns_each_section_with_its_id_and_title() {
        let root = TempDir::new().expect("should create");
        write_script(&root, "my_prs.js", "({ title: 'PRs', items: [] });");
        let app = app(&root);

        let (status, body) = call(&app, Method::GET, "/sections").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            body,
            serde_json::json!([{"id": "my_prs", "title": "My Prs"}])
        );
    }

    #[tokio::test]
    async fn fetching_returns_the_layout_json() {
        let root = TempDir::new().expect("should create");
        write_script(
            &root,
            "a.js",
            "({ title: 'A', items: [{ type: 'tile', title: 't' }] });",
        );
        let app = app(&root);

        let (status, body) = call(&app, Method::GET, "/sections/a").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["title"], "A");
        assert_eq!(body["items"][0]["type"], "tile");
    }

    #[tokio::test]
    async fn refreshing_reruns_the_script_and_returns_the_fresh_layout() {
        let root = TempDir::new().expect("should create");
        write_script(&root, "a.js", "({ title: 'Old', items: [] });");
        let app = app(&root);
        call(&app, Method::GET, "/sections/a").await;
        write_script(&root, "a.js", "({ title: 'New', items: [] });");

        let (status, body) = call(&app, Method::POST, "/sections/a/refresh").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["title"], "New");
    }

    #[tokio::test]
    async fn an_unknown_section_is_not_found() {
        let root = TempDir::new().expect("should create");
        let app = app(&root);

        let (status, body) = call(&app, Method::GET, "/sections/nope").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body["error"].is_string());
    }

    #[tokio::test]
    async fn a_failing_script_is_a_bad_gateway_that_keeps_the_old_layout() {
        let root = TempDir::new().expect("should create");
        write_script(&root, "a.js", "({ title: 'Good', items: [] });");
        let app = app(&root);
        call(&app, Method::GET, "/sections/a").await;
        write_script(&root, "a.js", "throw new Error('boom');");

        let (status, _) = call(&app, Method::POST, "/sections/a/refresh").await;
        let (after, body) = call(&app, Method::GET, "/sections/a").await;

        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert_eq!(after, StatusCode::OK);
        assert_eq!(body["title"], "Good");
    }
}
