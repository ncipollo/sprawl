//! The `--info` command: the topic directory or a single topic page.

use crate::feature::info;
use std::process::ExitCode;

/// Prints the directory (no topic) or one topic page. An unknown topic is
/// an error naming the valid ones.
pub fn run(topic: Option<&str>) -> ExitCode {
    let Some(name) = topic else {
        print!("{}", info::directory());
        return ExitCode::SUCCESS;
    };
    match info::topic(name) {
        Some(page) => {
            print!("{page}");
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("{}", unknown_topic_message(name));
            ExitCode::FAILURE
        }
    }
}

fn unknown_topic_message(name: &str) -> String {
    format!(
        "error: unknown topic \"{name}\": available topics: {}",
        info::names().join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_unknown_topic_message_names_every_valid_topic() {
        let message = unknown_topic_message("bogus");

        assert!(message.contains("\"bogus\""));
        for name in info::names() {
            assert!(message.contains(name), "missing {name}");
        }
    }
}
