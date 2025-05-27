use tanitc_ast::{expression_utils::BinaryOperation, Ast, ExpressionKind, ValueKind};

use tanitc_ident::Ident;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_ty::Type;

#[test]
fn variables_test() {
    let main_id = Ident::from("main".to_string());
    let pi_id = Ident::from("PI".to_string());
    let radian_id = Ident::from("radian".to_string());
    let ceil_id = Ident::from("ceil".to_string());

    const SRC_TEXT: &str = "\nfunc main() {\
                            \n    var const PI: f32\
                            \n    var radian = PI / 2.0\
                            \n    var mut ceil = radian as i32\
                            \n    ceil <<= 3\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).unwrap());

    let res = parser.parse_func_def().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    let res = if let Ast::FuncDef(node) = &res {
        assert!(node.identifier == main_id);
        assert!(node.parameters.is_empty());

        if let Type::Tuple(components) = &node.return_type.get_type() {
            assert!(components.is_empty());
        } else {
            panic!("Type expected to be an empty tuple");
        }

        node.body.as_ref()
    } else {
        panic!("res has to be \'FuncDef\'");
    };

    let res = if let Ast::Block(node) = res.unwrap().as_ref() {
        &node.statements
    } else {
        panic!("res has to be \'LScope\'");
    };

    if let Ast::VariableDef(node) = &res[0] {
        assert!(node.identifier == pi_id);
        assert!(!node.is_mutable);
        assert!(!node.is_global);
        assert_eq!(node.var_type.get_type(), Type::F32);
    } else {
        panic!("first statement has to be \'variable definition\'");
    }

    if let Ast::Expression(node) = &res[1] {
        let (lhs, rhs) = if let ExpressionKind::Binary {
            operation,
            lhs,
            rhs,
        } = &node.kind
        {
            assert_eq!(*operation, BinaryOperation::Assign);
            (lhs.as_ref(), rhs.as_ref())
        } else {
            panic!("Expected binary expression");
        };

        if let Ast::VariableDef(node) = lhs {
            assert!(node.identifier == radian_id);
            assert!(!node.is_global);
            assert!(!node.is_mutable);
        } else {
            panic!("Expected variable definition")
        }

        if let Ast::Expression(node) = rhs {
            if let ExpressionKind::Binary { operation, .. } = &node.kind {
                assert_eq!(*operation, BinaryOperation::Div);
            } else {
                panic!("expected binary expression")
            }
        } else {
            panic!("expected expression")
        }
    } else {
        panic!("second statement has to be \'variable definition\'");
    }

    if let Ast::Expression(node) = &res[2] {
        if let ExpressionKind::Binary {
            operation,
            lhs,
            rhs,
        } = &node.kind
        {
            assert_eq!(*operation, BinaryOperation::Assign);

            if let Ast::Expression(node) = lhs.as_ref() {
                let ExpressionKind::Conversion { lhs, ty } = &node.kind else {
                    panic!("Expected conversion, actually: {:#?}", node.kind);
                };

                if let Ast::VariableDef(node) = lhs.as_ref() {
                    assert!(node.identifier == ceil_id);
                    assert!(!node.is_global);
                    assert!(!node.is_mutable);
                } else {
                    panic!("Expected variable definition")
                }

                assert!(ty.get_type() == Type::I32);
            }

            let expr = if let Ast::Expression(node) = rhs.as_ref() {
                node
            } else {
                panic!("rhs expected to be \'Expression\'");
            };

            if let ExpressionKind::Conversion { lhs, ty } = &expr.kind {
                if let Ast::Value(node) = lhs.as_ref() {
                    if let ValueKind::Identifier(id) = &node.kind {
                        assert!(*id == radian_id)
                    }
                } else {
                    panic!("rhs has to be \'Expression\'");
                };

                assert!(matches!(ty.get_type(), Type::I32));
            } else {
                panic!("Expected binary expression");
            }
        }
    } else {
        panic!("third statement has to be \'expression\'");
    }
}
