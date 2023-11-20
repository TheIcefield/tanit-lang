#[test]
fn lexer_test() {
    use garnet_script::lexer::{Lexer, Location, Token, TokenType};

    static SRC: &str = "hello func let + 65 -= <<\n struct alpha";

    let mut lexer = Lexer::from_string(SRC, true).unwrap();

    assert_eq!(
        lexer.get(),
        Token::new(
            TokenType::Identifier("hello".to_string()),
            Location { row: 1, col: 2 }
        )
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::KwFunc, Location { row: 1, col: 8 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::KwLet, Location { row: 1, col: 13 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::Plus, Location { row: 1, col: 18 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::Integer(65usize), Location { row: 1, col: 19 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::SubAssign, Location { row: 1, col: 23 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::LShift, Location { row: 1, col: 27 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::EndOfLine, Location { row: 2, col: 1 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::KwStruct, Location { row: 2, col: 3 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(
            TokenType::Identifier("alpha".to_string()),
            Location { row: 2, col: 10 }
        )
    );
}
