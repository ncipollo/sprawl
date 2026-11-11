//! Parsing the command line and dispatching to the right command.

use crate::cli::info;
use crate::feature::config;
use crate::feature::section::Section;
use crate::feature::section::source::{LocalSource, SectionSource};
use crate::feature::server;
use crate::network::client::RemoteSource;
use crate::network::server as http;
use crate::ui::app;
use clap::{Args, Parser, Subcommand};
use std::net::{IpAddr, SocketAddr};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

/// What was I doing? Where did it go?
#[derive(Debug, Parser)]
#[command(
    name = "sprawl",
    version,
    about,
    args_conflicts_with_subcommands = true
)]
struct Cli {
    /// Print documentation and exit. Bare --info lists the available
    /// topics; --info <TOPIC> prints one.
    #[arg(long, value_name = "TOPIC", num_args = 0..=1)]
    info: Option<Option<String>>,

    /// Open the app against a sprawl server (started with `sprawl serve`)
    /// instead of running config scripts locally, e.g. http://host:7878.
    #[arg(long, value_name = "URL")]
    connect: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run headless: render every section script on an interval into JSON
    /// under ~/.sprawl/output, and serve the results over HTTP.
    Serve(ServeArgs),
}

#[derive(Debug, Args)]
struct ServeArgs {
    /// Seconds between renders of every section.
    #[arg(long, value_name = "SECONDS", default_value_t = server::DEFAULT_INTERVAL_SECS, value_parser = clap::value_parser!(u64).range(1..))]
    interval: u64,

    /// Address to listen on. Defaults to loopback: the server has no
    /// authentication, so only expose it on networks you trust.
    #[arg(long, value_name = "ADDRESS", default_value = "127.0.0.1")]
    host: IpAddr,

    /// Port to listen on.
    #[arg(long, value_name = "PORT", default_value_t = http::DEFAULT_PORT)]
    port: u16,
}

/// Parses the real command line and runs sprawl.
pub fn run() -> ExitCode {
    let Cli {
        info,
        connect,
        command,
    } = Cli::parse();
    if let Some(topic) = info {
        return info::run(topic.as_deref());
    }
    if let Some(Command::Serve(args)) = command {
        return serve(args);
    }
    if let Some(url) = connect {
        return open_remote(&url);
    }
    app::run(
        config::startup_sections(),
        LocalSource::shared(),
        config::NO_SECTIONS_MESSAGE.to_string(),
    );
    ExitCode::SUCCESS
}

/// Opens the gui against a server. A bad URL is a usage error; a server that
/// cannot be reached still opens the window, showing why.
fn open_remote(url: &str) -> ExitCode {
    let source = match RemoteSource::new(url) {
        Ok(source) => source,
        Err(error) => return failure(error),
    };
    let (sections, message) = match source.sections() {
        Ok(sections) => (sections, "The server has no sections.".to_string()),
        Err(error) => (Vec::<Section>::new(), error.to_string()),
    };
    let source: Arc<dyn SectionSource> = Arc::new(source);
    app::run(sections, source, message);
    ExitCode::SUCCESS
}

fn serve(args: ServeArgs) -> ExitCode {
    let service = match server::start(Duration::from_secs(args.interval)) {
        Ok(service) => service,
        Err(error) => return failure(error),
    };
    match http::serve(service, SocketAddr::new(args.host, args.port)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => failure(error),
    }
}

fn failure(error: impl std::fmt::Display) -> ExitCode {
    eprintln!("sprawl: {error}");
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn the_command_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn no_arguments_means_open_the_gui() {
        let cli = Cli::try_parse_from(["sprawl"]).expect("should parse");

        assert_eq!(cli.info, None);
    }

    #[test]
    fn bare_info_asks_for_the_directory() {
        let cli = Cli::try_parse_from(["sprawl", "--info"]).expect("should parse");

        assert_eq!(cli.info, Some(None));
    }

    #[test]
    fn info_with_a_topic_asks_for_that_page() {
        let cli = Cli::try_parse_from(["sprawl", "--info", "scripts"]).expect("should parse");

        assert_eq!(cli.info, Some(Some("scripts".to_string())));
    }

    #[test]
    fn serve_defaults_its_interval() {
        let cli = Cli::try_parse_from(["sprawl", "serve"]).expect("should parse");

        let Some(Command::Serve(args)) = cli.command else {
            panic!("expected serve")
        };
        assert_eq!(args.interval, server::DEFAULT_INTERVAL_SECS);
        assert_eq!(args.port, http::DEFAULT_PORT);
        assert!(args.host.is_loopback());
    }

    #[test]
    fn serve_takes_an_interval() {
        let cli =
            Cli::try_parse_from(["sprawl", "serve", "--interval", "30"]).expect("should parse");

        let Some(Command::Serve(args)) = cli.command else {
            panic!("expected serve")
        };
        assert_eq!(args.interval, 30);
    }

    #[test]
    fn connect_takes_a_url() {
        let cli =
            Cli::try_parse_from(["sprawl", "--connect", "http://host:1"]).expect("should parse");

        assert_eq!(cli.connect.as_deref(), Some("http://host:1"));
    }

    #[test]
    fn connect_and_serve_cannot_be_combined() {
        assert!(Cli::try_parse_from(["sprawl", "--connect", "http://h", "serve"]).is_err());
    }

    #[test]
    fn no_arguments_means_local_mode() {
        let cli = Cli::try_parse_from(["sprawl"]).expect("should parse");

        assert_eq!(cli.connect, None);
    }

    #[test]
    fn unexpected_arguments_are_an_error() {
        assert!(Cli::try_parse_from(["sprawl", "extra"]).is_err());
    }
}
