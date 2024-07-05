use tanit::ast::{expressions::Expression, identifiers::Identifier, values::Value, Ast};
use tanit::lexer::Lexem;

#[test]
fn str_conversion_test() {
    use std::str::FromStr;

    let id = Identifier::from_str("hello").unwrap();

    if let Identifier::Common(id) = &id {
        assert_eq!(id, "hello");
    } else {
        panic!("expected common identifier");
    }
}

#[test]
fn expr_conversion_test() {
    let expression = Expression::Binary {
        operation: Lexem::Dcolon,
        lhs: Box::new(Ast::Value {
            node: Value::Identifier(Identifier::Common("hello".to_string())),
        }),
        rhs: Box::new(Ast::Expression {
            node: Box::new(Expression::Binary {
                operation: Lexem::Dcolon,
                lhs: Box::new(Ast::Value {
                    node: Value::Identifier(Identifier::Common("my".to_string())),
                }),
                rhs: Box::new(Ast::Value {
                    node: Value::Identifier(Identifier::Common("world".to_string())),
                }),
            }),
        }),
    };

    let expected = vec!["hello".to_string(), "my".to_string(), "world".to_string()];

    if let Identifier::Complex(ids) = Identifier::from_expr(&expression).unwrap() {
        assert_eq!(ids, expected);
    } else {
        panic!("expecred complex identifier");
    }
}
