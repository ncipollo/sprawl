//! The `example` topic: a complete, working script, embedded from the same
//! source the first-run scaffold writes.

use crate::feature::config::examples;

pub fn render() -> String {
    format!(
        "EXAMPLE\n\
         The my_prs.js script scaffolded on first run. It shells out to\n\
         the GitHub CLI, parses the JSON response, and maps each pull\n\
         request to a tile:\n\n{}",
        examples::my_prs()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_embeds_the_scaffolded_example_verbatim() {
        assert!(render().contains(&examples::my_prs()));
    }
}
