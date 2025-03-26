use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::XmlWriter;

#[test]
fn use_test() {
    const SRC_TEXT: &str = "use hello::world";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let use_node = parser.parse_use().unwrap();

    {
        const EXPECTED: &str = "\n<use name=\"hello::world\"/>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        use_node.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }
}

#[test]
fn use_all_test() {
    const SRC_TEXT: &str = "use Self::mod::*";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let use_node = parser.parse_use().unwrap();

    {
        const EXPECTED: &str = "\n<use name=\"Self::mod::*\"/>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        use_node.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }
}

#[test]
fn use_all_wrong_test() {
    const SRC_TEXT: &str = "use Self::mod::*::hi";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    parser
        .parse_use()
        .err()
        .expect("Expected fail on parse_use");
}
