//! The HTTP client for connecting to a sprawl server. Implements the
//! feature layer's [`SectionSource`], so the ui is unaware it is remote.

use crate::feature::script::error::ScriptError;
use crate::feature::script::schema::SectionConfig;
use crate::feature::section::source::SectionSource;
use crate::feature::section::{Section, SectionKey};
use crate::feature::server::service::SectionSummary;
use reqwest::Url;
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Duration;

/// A refresh runs the section's script on the server, which can be slow.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Fetches sections from a sprawl server.
pub struct RemoteSource {
    base: Url,
    client: Client,
}

impl RemoteSource {
    /// Connects to the server at `url`, e.g. `http://host:7878`. Nothing is
    /// sent until the first request.
    pub fn new(url: &str) -> Result<Self, ScriptError> {
        let base = Url::parse(url)
            .ok()
            .filter(|url| matches!(url.scheme(), "http" | "https") && url.has_host())
            .ok_or_else(|| remote(format!("{url:?} is not an http(s) server URL")))?;
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| remote(error.to_string()))?;
        Ok(Self { base, client })
    }

    /// Lists the server's sections.
    pub fn sections(&self) -> Result<Vec<Section>, ScriptError> {
        let summaries: Vec<SectionSummary> =
            self.send(self.client.get(self.url(&["sections"])?))?;
        Ok(summaries
            .into_iter()
            .map(|summary| Section::remote(summary.id, summary.title))
            .collect())
    }

    fn url(&self, segments: &[&str]) -> Result<Url, ScriptError> {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|_| remote("the server URL cannot be a base".to_string()))?
            .pop_if_empty()
            .extend(segments);
        Ok(url)
    }

    fn id_of(key: &SectionKey) -> Result<&str, ScriptError> {
        match key {
            SectionKey::Remote(id) => Ok(id),
            SectionKey::Script(_) => Err(remote("not a server section".to_string())),
        }
    }

    fn send<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T, ScriptError> {
        let response = request.send().map_err(|error| self.unreachable(error))?;
        let response = check_status(response)?;
        response
            .json()
            .map_err(|error| remote(format!("the server sent an unreadable response: {error}")))
    }

    fn unreachable(&self, error: reqwest::Error) -> ScriptError {
        let reason = if error.is_timeout() {
            "timed out".to_string()
        } else {
            error.without_url().to_string()
        };
        remote(format!("could not reach {}: {reason}", self.base))
    }
}

impl SectionSource for RemoteSource {
    fn load(&self, key: &SectionKey) -> Result<SectionConfig, ScriptError> {
        let url = self.url(&["sections", Self::id_of(key)?])?;
        self.send(self.client.get(url))
    }

    fn refresh(&self, key: &SectionKey) -> Result<SectionConfig, ScriptError> {
        let url = self.url(&["sections", Self::id_of(key)?, "refresh"])?;
        self.send(self.client.post(url))
    }

    fn refreshes_on_select(&self) -> bool {
        true
    }
}

fn remote(message: String) -> ScriptError {
    ScriptError::Remote(message)
}

/// Turns a non-success response into the server's `{"error": ...}` message.
fn check_status(response: Response) -> Result<Response, ScriptError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let message = response
        .json::<Value>()
        .ok()
        .and_then(|body| body["error"].as_str().map(str::to_string))
        .unwrap_or_else(|| "no details".to_string());
    Err(remote(format!("the server answered {status}: {message}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::script::shell::fake::FakeShell;
    use crate::feature::server::renderer::SectionRenderer;
    use crate::feature::server::service::SectionService;
    use crate::network::server::{AppState, build_router};
    use std::fs;
    use std::net::TcpListener;
    use std::sync::Arc;
    use std::thread;
    use tempfile::TempDir;

    struct TestServer {
        root: TempDir,
        url: String,
    }

    impl TestServer {
        fn start() -> Self {
            let root = TempDir::new().expect("should create");
            let scripts = root.path().join("default");
            fs::create_dir_all(&scripts).expect("should create");
            let renderer = SectionRenderer::new(
                scripts,
                root.path().join("output"),
                Arc::new(FakeShell::outputting("")),
            );
            let router = build_router(AppState::new(SectionService::new(renderer)));
            let listener = TcpListener::bind("127.0.0.1:0").expect("should bind");
            listener.set_nonblocking(true).expect("should configure");
            let url = format!("http://{}", listener.local_addr().expect("address"));
            thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().expect("runtime");
                runtime.block_on(async {
                    let listener = tokio::net::TcpListener::from_std(listener).expect("listener");
                    axum::serve(listener, router).await.ok();
                });
            });
            Self { root, url }
        }

        fn write_script(&self, name: &str, title: &str) {
            let source = format!("({{ title: \"{title}\", items: [] }});");
            fs::write(self.root.path().join("default").join(name), source).expect("should write");
        }
    }

    #[test]
    fn the_section_list_becomes_remote_sections() {
        let server = TestServer::start();
        server.write_script("my_prs.js", "PRs");
        let source = RemoteSource::new(&server.url).expect("should connect");

        let sections = source.sections().expect("should list");

        assert_eq!(
            sections,
            vec![Section::remote("my_prs".to_string(), "My Prs".to_string())]
        );
    }

    #[test]
    fn load_fetches_the_layout_and_refresh_reruns_the_script() {
        let server = TestServer::start();
        server.write_script("a.js", "Old");
        let source = RemoteSource::new(&server.url).expect("should connect");
        let key = SectionKey::Remote("a".to_string());
        assert_eq!(source.load(&key).expect("should load").title, "Old");
        server.write_script("a.js", "New");

        assert_eq!(source.load(&key).expect("should load").title, "Old");
        assert_eq!(source.refresh(&key).expect("should refresh").title, "New");
    }

    #[test]
    fn a_server_error_surfaces_the_servers_message() {
        let server = TestServer::start();
        let source = RemoteSource::new(&server.url).expect("should connect");

        let error = source
            .load(&SectionKey::Remote("missing".to_string()))
            .expect_err("should fail");

        let message = error.to_string();
        assert!(message.contains("404"), "{message}");
        assert!(message.contains("no section named"), "{message}");
    }

    #[test]
    fn an_unreachable_server_names_the_address() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("should bind");
        let url = format!("http://{}", listener.local_addr().expect("address"));
        drop(listener);
        let source = RemoteSource::new(&url).expect("should connect");

        let error = source.sections().expect_err("should fail").to_string();

        assert!(error.contains("could not reach"), "{error}");
        assert!(error.contains(&url[7..]), "{error}");
    }

    #[test]
    fn ids_with_awkward_characters_are_escaped_into_one_path_segment() {
        let source = RemoteSource::new("http://localhost:1/").expect("should connect");

        let url = source.url(&["sections", "a b/../c"]).expect("should build");

        assert_eq!(url.path(), "/sections/a%20b%2F..%2Fc");
    }

    #[test]
    fn only_http_urls_are_accepted() {
        assert!(RemoteSource::new("ftp://host").is_err());
        assert!(RemoteSource::new("not a url").is_err());
    }

    #[test]
    fn a_local_key_is_not_a_server_section() {
        let source = RemoteSource::new("http://localhost:1").expect("should connect");

        let result = source.load(&SectionKey::Script("a.js".into()));

        assert!(result.is_err());
    }
}
