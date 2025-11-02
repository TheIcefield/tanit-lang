pub mod listener;
pub mod messages;

pub use messages::Message;

pub type Error = messages::Message;
pub type Warning = messages::Message;

pub type Errors = Vec<Error>;
pub type Warnings = Vec<Warning>;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tanitc_lexer::location::Location;

    use crate::Message;

    #[test]
    fn message_test() {
        const FILE: &str = "main.tt";
        const MESSAGE: &str = "Some message";

        let location = Location::new(&PathBuf::from(FILE));
        let msg = Message::new(&location, MESSAGE);

        assert_eq!(msg.to_string(), "main.tt:1:1: Some message");
    }
}
