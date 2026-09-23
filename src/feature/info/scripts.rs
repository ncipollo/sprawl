//! The `scripts` topic: how a section script is evaluated and what it may
//! do.

pub fn render() -> String {
    "SCRIPTS\n\
     A config script is plain JavaScript evaluated in a sandbox. The\n\
     script's completion value — its final expression statement — is\n\
     the section object. End every script with an expression like:\n\n\
     \x20 ({ title: \"My Section\", items });\n\n\
     The object has two fields: title, the section's sidebar name, and\n\
     items, an array of typed items (see --info tiles, --info groups).\n\n\
     THE SHELL FUNCTION\n\
     shell(command) runs command through /bin/sh -c and returns its\n\
     standard output as a string.\n\
     shell(program, args) runs program directly with an array of\n\
     string arguments and no shell interpretation — prefer this form\n\
     when an argument contains quotes or newlines.\n\
     When the command exits non-zero, shell throws an Error carrying\n\
     the exit status and the first line of stderr. Uncaught, that\n\
     fails the whole section; catch it to substitute your own items.\n\n\
     THE SANDBOX\n\
     Scripts have no fetch, no file system, no network, no timers, and\n\
     no imports. The shell function is the only way out. Standard\n\
     JavaScript built-ins (JSON, Math, String, Array) are available.\n\n\
     See also: --info tiles, --info groups, --info example\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_documents_the_completion_value() {
        let page = render();

        assert!(page.contains("completion value"));
        assert!(page.contains("({ title: \"My Section\", items });"));
    }

    #[test]
    fn page_documents_both_shell_forms() {
        let page = render();

        assert!(page.contains("shell(command)"));
        assert!(page.contains("shell(program, args)"));
    }

    #[test]
    fn page_documents_the_sandbox() {
        assert!(render().contains("no fetch"));
    }
}
