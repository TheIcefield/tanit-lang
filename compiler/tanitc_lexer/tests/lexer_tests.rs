use tanitc_lexer::{
    location::Location,
    token::{Lexem, Token},
    Lexer,
};

#[test]
fn lexer_test() {
    const SRC_TEXT: &str = "hello func let + 65 -= <<\n struct alpha";

    let mut lexer = Lexer::from_text(SRC_TEXT).unwrap();

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Identifier("hello".to_string()),
            Location { row: 1, col: 2 }
        )
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwFunc, Location { row: 1, col: 8 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwLet, Location { row: 1, col: 13 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::Plus, Location { row: 1, col: 18 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Integer("65".to_string()),
            Location { row: 1, col: 19 }
        )
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::SubAssign, Location { row: 1, col: 23 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::LShift, Location { row: 1, col: 27 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwStruct, Location { row: 2, col: 3 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Identifier("alpha".to_string()),
            Location { row: 2, col: 10 }
        )
    );
}

#[test]
fn lexer_without_ignore_test() {
    const SRC_TEXT: &str = "hello func let + 65 -= <<\n struct alpha";

    let mut lexer = Lexer::from_text(SRC_TEXT).unwrap();

    lexer.ignores_nl = false;

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Identifier("hello".to_string()),
            Location { row: 1, col: 2 }
        )
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwFunc, Location { row: 1, col: 8 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwLet, Location { row: 1, col: 13 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::Plus, Location { row: 1, col: 18 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Integer("65".to_string()),
            Location { row: 1, col: 19 }
        )
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::SubAssign, Location { row: 1, col: 23 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::LShift, Location { row: 1, col: 27 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::EndOfLine, Location { row: 2, col: 1 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwStruct, Location { row: 2, col: 3 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Identifier("alpha".to_string()),
            Location { row: 2, col: 10 }
        )
    );
}
