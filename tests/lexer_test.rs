
#[test]
fn lexer_test() {
    use garnet_script::lexer::{Lexer, Token, TokenType, Location};

    static SRC: &str = "hello func let + 65 -= <<\n struct alpha";

    let mut lexer = Lexer::from_string(SRC, true).unwrap();

    assert_eq!(
        lexer.get(),
        Token::new(
            TokenType::Identifier("hello".to_string()),
            Location { row: 0, col: 1 }
        )
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::KwFunc, Location { row: 0, col: 7 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::KwLet, Location { row: 0, col: 12 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::Plus, Location { row: 0, col: 17 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::Integer(65usize), Location { row: 0, col: 18 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::SubAssign, Location { row: 0, col: 22 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::LShift, Location { row: 0, col: 26 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::EndOfLine, Location { row: 1, col: 0 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(TokenType::KwStruct, Location { row: 1, col: 2 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(
            TokenType::Identifier("alpha".to_string()),
            Location { row: 1, col: 9 }
        )
    );
}
