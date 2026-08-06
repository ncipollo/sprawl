/// The message the root view renders.
pub fn message() -> String {
    "Hello, world!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_greets_the_world() {
        assert_eq!(message(), "Hello, world!");
    }
}
