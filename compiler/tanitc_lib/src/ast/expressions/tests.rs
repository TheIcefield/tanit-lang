use super::{Expression, ExpressionType};
use crate::ast::Ast;

use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_ty::Type;

#[test]
fn conversion_test() {
    use crate::ast::values::{Value, ValueType};

    const SRC_TEXT: &str = "45 as f32";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    if let Ast::Expression(node) = Expression::parse(&mut parser).unwrap() {
        if let ExpressionType::Conversion { lhs, ty } = &node.expr {
            assert!(matches!(
                lhs.as_ref(),
                Ast::Value(Value {
                    value: ValueType::Integer(45),
                    ..
                })
            ));

            assert!(matches!(ty.get_type(), Type::F32))
        } else {
            panic!("Expected binary expression");
        }
    } else {
        panic!("Expected expression");
    };
}
