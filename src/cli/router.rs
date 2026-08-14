//! Parsing the command line and dispatching to the right command.

use crate::cli::info;
use crate::ui::app;
use clap::Parser;
use std::process::ExitCode;

/// What was I doing? Where did it go?
#[derive(Debug, Parser)]
#[command(name = "sprawl", version, about)]
struct Cli {
    /// Print documentation and exit. Bare --info lists the available
    /// topics; --info <TOPIC> prints one.
    #[arg(long, value_name = "TOPIC", num_args = 0..=1)]
    info: Option<Option<String>>,
}

/// Parses the real command line and runs sprawl.
pub fn run() -> ExitCode {
    let Cli { info } = Cli::parse();
    if let Some(topic) = info {
        return info::run(topic.as_deref());
    }
    app::run();
    ExitCode::SUCCESS
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
    fn unexpected_arguments_are_an_error() {
        assert!(Cli::try_parse_from(["sprawl", "extra"]).is_err());
    }
}
