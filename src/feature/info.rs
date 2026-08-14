//! Topic registry for `--info`, so AI agents can walk the documentation
//! one topic at a time instead of reading a single giant page.

pub mod example;
pub mod scripts;
pub mod tiles;
pub mod usage;

/// One documentation topic: a name to request, a one-line summary for the
/// directory, and the page itself.
pub struct Topic {
    pub name: &'static str,
    pub summary: &'static str,
    render: fn() -> String,
}

/// Every topic, in directory order.
pub const TOPICS: &[Topic] = &[
    Topic {
        name: "usage",
        summary: "Invoking sprawl, the config directory, caching",
        render: usage::render,
    },
    Topic {
        name: "scripts",
        summary: "Writing section scripts, the shell function, the sandbox",
        render: scripts::render,
    },
    Topic {
        name: "tiles",
        summary: "The tile item schema and badge colors",
        render: tiles::render,
    },
    Topic {
        name: "example",
        summary: "A complete script: GitHub pull requests as tiles",
        render: example::render,
    },
];

/// The page shown for a bare `--info`: every topic and how to read one.
pub fn directory() -> String {
    let mut page = String::from("TOPICS\n");
    for topic in TOPICS {
        page.push_str(&format!("  {:<10}{}\n", topic.name, topic.summary));
    }
    page.push_str("\nrun: sprawl --info <topic>\n");
    page
}

/// The page for `name`, if it is a known topic.
pub fn topic(name: &str) -> Option<String> {
    TOPICS
        .iter()
        .find(|topic| topic.name == name)
        .map(|topic| (topic.render)())
}

/// Every topic name, for error messages.
pub fn names() -> Vec<&'static str> {
    TOPICS.iter().map(|topic| topic.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_lists_every_topic() {
        let directory = directory();

        for topic in TOPICS {
            assert!(directory.contains(topic.name), "missing {}", topic.name);
            assert!(
                directory.contains(topic.summary),
                "missing {}",
                topic.summary
            );
        }
    }

    #[test]
    fn every_directory_topic_resolves() {
        for name in names() {
            assert!(topic(name).is_some(), "unresolvable topic {name}");
        }
    }

    #[test]
    fn topic_pages_are_non_empty() {
        for name in names() {
            assert!(!topic(name).expect("should resolve").trim().is_empty());
        }
    }

    #[test]
    fn an_unknown_topic_does_not_resolve() {
        assert!(topic("bogus").is_none());
    }

    #[test]
    fn directory_stays_short() {
        assert!(
            directory().lines().count() <= 15,
            "directory grew past a quick scan"
        );
    }
}
