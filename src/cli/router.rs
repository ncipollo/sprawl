//! Parsing the command line and dispatching to the right command.

use crate::cli::info;
use crate::feature::server;
use crate::ui::app;
use clap::{Parser, Subcommand};
use std::process::ExitCode;
use std::time::Duration;

/// What was I doing? Where did it go?
#[derive(Debug, Parser)]
#[command(name = "sprawl", version, about)]
struct Cli {
    /// Print documentation and exit. Bare --info lists the available
    /// topics; --info <TOPIC> prints one.
    #[arg(long, value_name = "TOPIC", num_args = 0..=1)]
    info: Option<Option<String>>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run headless: evaluate every section script on an interval and
    /// write each rendered section to JSON under ~/.sprawl/output.
    Serve {
        /// Seconds between renders of every section.
        #[arg(long, value_name = "SECONDS", default_value_t = server::DEFAULT_INTERVAL_SECS, value_parser = clap::value_parser!(u64).range(1..))]
        interval: u64,
    },
}

/// Parses the real command line and runs sprawl.
pub fn run() -> ExitCode {
    let Cli { info, command } = Cli::parse();
    if let Some(topic) = info {
        return info::run(topic.as_deref());
    }
    if let Some(Command::Serve { interval }) = command {
        return serve(interval);
    }
    app::run();
    ExitCode::SUCCESS
}

fn serve(interval: u64) -> ExitCode {
    match server::run(Duration::from_secs(interval)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sprawl: {error}");
            ExitCode::FAILURE
        }
    }
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

        let Some(Command::Serve { interval }) = cli.command else {
            panic!("expected serve")
        };
        assert_eq!(interval, server::DEFAULT_INTERVAL_SECS);
    }

    #[test]
    fn serve_takes_an_interval() {
        let cli =
            Cli::try_parse_from(["sprawl", "serve", "--interval", "30"]).expect("should parse");

        let Some(Command::Serve { interval }) = cli.command else {
            panic!("expected serve")
        };
        assert_eq!(interval, 30);
    }

    #[test]
    fn unexpected_arguments_are_an_error() {
        assert!(Cli::try_parse_from(["sprawl", "extra"]).is_err());
    }
}
