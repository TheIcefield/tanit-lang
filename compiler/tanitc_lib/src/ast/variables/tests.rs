use crate::ast::{
    expressions::ExpressionType, functions::FunctionDef, identifiers::Identifier, types::Type,
    values::ValueType, Ast,
};
use crate::parser::{lexer::Lexer, token::Lexem, Parser};

#[test]
fn variables_test() {
    use std::str::FromStr;

    let radian_var_id = Identifier::from_str("radian").unwrap();
    let i32_type_id = Identifier::from_str("i32").unwrap();

    const SRC_TEXT: &str = "\nfunc main()
                            \n{\
                            \n    let const PI: f32\
                            \n    let radian = PI / 2.0\
                            \n    let mut ceil = radian as i32\
                            \n    ceil <<= 3\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).unwrap());

    let res = FunctionDef::parse(&mut parser).unwrap();

    let res = if let Ast::FuncDef(node) = &res {
        assert!(node.identifier == Identifier::from_str("main").unwrap());
        assert!(node.parameters.is_empty());

        if let Type::Tuple { components } = &node.return_type {
            assert!(components.is_empty());
        } else {
            panic!("Type expected to be an empty tuple");
        }

        node.body.as_ref()
    } else {
        panic!("res has to be \'FuncDef\'");
    };

    let res = if let Ast::Scope(node) = res.unwrap().as_ref() {
        &node.statements
    } else {
        panic!("res has to be \'LScope\'");
    };

    if let Ast::VariableDef(node) = &res[0] {
        assert!(node.identifier == Identifier::from_str("PI").unwrap());
        assert!(!node.is_mutable);
        assert!(!node.is_global);
        assert_eq!(node.var_type, Type::F32);
    } else {
        panic!("first statement has to be \'variable definition\'");
    }

    if let Ast::Expression(node) = &res[1] {
        let (lhs, rhs) = if let ExpressionType::Binary {
            operation,
            lhs,
            rhs,
        } = &node.expr
        {
            assert_eq!(*operation, Lexem::Assign);
            (lhs.as_ref(), rhs.as_ref())
        } else {
            panic!("Expected binary expression");
        };

        if let Ast::VariableDef(node) = lhs {
            assert!(node.identifier == radian_var_id);
            assert!(!node.is_global);
            assert!(!node.is_mutable);
        } else {
            panic!("Expected variable definition")
        }

        if let Ast::Expression(node) = rhs {
            if let ExpressionType::Binary { operation, .. } = &node.expr {
                assert_eq!(*operation, Lexem::Slash);
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
        if let ExpressionType::Binary {
            operation,
            lhs,
            rhs,
        } = &node.expr
        {
            assert_eq!(*operation, Lexem::Assign);

            if let Ast::Expression(node) = lhs.as_ref() {
                let (lhs, rhs) = if let ExpressionType::Binary {
                    operation,
                    lhs,
                    rhs,
                } = &node.expr
                {
                    assert_eq!(*operation, Lexem::KwAs);
                    (lhs.as_ref(), rhs.as_ref())
                } else {
                    panic!("Binary expression expected")
                };

                if let Ast::VariableDef(node) = lhs {
                    assert!(node.identifier == Identifier::from_str("ceil").unwrap());
                    assert!(!node.is_global);
                    assert!(!node.is_mutable);
                } else {
                    panic!("Expected variable definition")
                }

                if let Ast::Value(node) = rhs {
                    if let ValueType::Identifier(id) = &node.value {
                        assert!(*id == i32_type_id);
                    } else {
                        panic!("Expected identifier")
                    }
                } else {
                    panic!("Expected value")
                }
            }

            let expr = if let Ast::Expression(node) = rhs.as_ref() {
                node
            } else {
                panic!("rhs expected to be \'Expression\'");
            };

            if let ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } = &expr.expr
            {
                assert_eq!(*operation, Lexem::KwAs);

                if let Ast::Value(node) = lhs.as_ref() {
                    if let ValueType::Identifier(id) = &node.value {
                        assert!(*id == Identifier::from_str("radian").unwrap())
                    }
                } else {
                    panic!("rhs has to be \'Expression\'");
                };

                assert!(matches!(rhs.as_ref(), Ast::Type(Type::I32)));
            } else {
                panic!("Expected binary expression");
            }
        }
    } else {
        panic!("third statement has to be \'expression\'");
    }
}
