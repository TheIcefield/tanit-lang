use tanitc_ast::{Ast, ExpressionKind};

use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_ty::Type;

#[test]
fn conversion_test() {
    use tanitc_ast::{Value, ValueKind};

    const SRC_TEXT: &str = "45 as f32";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    if let Ast::Expression(node) = parser.parse_expression().unwrap() {
        if let ExpressionKind::Conversion { lhs, ty } = &node.kind {
            assert!(matches!(
                lhs.as_ref(),
                Ast::Value(Value {
                    kind: ValueKind::Integer(45),
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
