use super::{Expression, ExpressionType};
use crate::ast::{types::Type, Ast};
use crate::parser::{token::Lexem, Parser};

#[test]
fn conversion_test() {
    use crate::ast::values::{Value, ValueType};
    use crate::parser::lexer::Lexer;

    const SRC_TEXT: &str = "45 as f32";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    if let Ast::Expression(node) = Expression::parse(&mut parser).unwrap() {
        if let ExpressionType::Binary {
            operation,
            lhs,
            rhs,
        } = &node.expr
        {
            assert_eq!(*operation, Lexem::KwAs);

            assert!(matches!(
                lhs.as_ref(),
                Ast::Value(Value {
                    value: ValueType::Integer(45),
                    ..
                })
            ));

            assert!(matches!(rhs.as_ref(), Ast::Type(Type::F32)))
        } else {
            panic!("Expected binary expression");
        }
    } else {
        panic!("Expected expression");
    };
}
