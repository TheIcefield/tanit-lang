use tanit::{
    ast::{structs::EnumField, types::Type},
    lexer::TokenType,
};

#[test]
fn module_test() {
    use tanit::ast;
    use tanit::{error_listener, lexer, parser};

    static SRC_PATH: &str = "./examples/modules.tt";

    let lexer = lexer::Lexer::from_file(SRC_PATH, false);

    assert_eq!(lexer.is_ok(), true);

    let lexer = lexer.unwrap();

    let error_listener = error_listener::ErrorListener::new();

    let mut parser = parser::Parser::new(lexer, error_listener);

    let res = tanit::ast::modules::ModuleNode::parse_def(&mut parser).unwrap();

    let res = if let ast::Ast::ModuleDef { node } = &res {
        assert_eq!(node.identifier, "M1");
        &node.body
    } else {
        panic!("res should be \'ModuleDef\'");
    };

    let res = if let ast::Ast::Scope { node } = res.as_ref() {
        &node.statements[0]
    } else {
        panic!("res should be \'global scope\'");
    };

    if let ast::Ast::ModuleDef { node } = res {
        assert_eq!(node.identifier, "M2");
    } else {
        panic!("res should be \'ModuleDef\'");
    };
}

#[test]
fn struct_test() {
    use tanit::ast;
    use tanit::{error_listener, lexer, parser};

    static SRC_PATH: &str = "./examples/structs.tt";

    let lexer = lexer::Lexer::from_file(SRC_PATH, false);

    assert_eq!(lexer.is_ok(), true);

    let lexer = lexer.unwrap();

    let error_listener = error_listener::ErrorListener::new();

    let mut parser = parser::Parser::new(lexer, error_listener);

    let res = tanit::ast::structs::StructNode::parse_def(&mut parser).unwrap();

    if let ast::Ast::StructDef { node } = res {
        assert_eq!(node.identifier, String::from("S1"));

        let field_type = node.fields.get("f1").unwrap();
        assert!(matches!(field_type, Type::I32));

        let field_type = node.fields.get("f2").unwrap();

        if let Type::Template {
            identifier,
            arguments,
        } = &field_type
        {
            assert_eq!(identifier, "Vec");
            assert_eq!(arguments.len(), 1);
            assert!(matches!(arguments[0], Type::I32));
        } else {
            panic!("wrong type");
        }
    } else {
        panic!("res should be \'StructDef\'");
    };
}

#[test]
fn enum_test() {
    use tanit::ast;
    use tanit::{error_listener, lexer, parser};

    static SRC_PATH: &str = "./examples/structs.tt";

    let lexer = lexer::Lexer::from_file(SRC_PATH, false);

    assert_eq!(lexer.is_ok(), true);

    let lexer = lexer.unwrap();

    let error_listener = error_listener::ErrorListener::new();

    let mut parser = parser::Parser::new(lexer, error_listener);

    tanit::ast::structs::StructNode::parse_def(&mut parser).unwrap();

    let res = tanit::ast::structs::EnumNode::parse_def(&mut parser).unwrap();

    if let ast::Ast::EnumDef { node } = &res {
        assert_eq!(node.identifier, "E1");

        assert!(matches!(node.fields.get("f1"), Some(&EnumField::Common)));

        if let EnumField::TupleLike(components) = node.fields.get("f2").unwrap() {
            assert_eq!(components.len(), 2);
            assert!(matches!(components[0], Type::I32));
            assert!(matches!(components[1], Type::I32));
        } else {
            panic!("wrong type");
        }

        let field = node.fields.get("f3").unwrap();
        if let EnumField::StructLike(components) = &field {
            assert_eq!(components.len(), 2);
            assert!(matches!(components.get("f1"), Some(&Type::I32)));
            assert!(matches!(components.get("f2"), Some(&Type::F32)));
        } else {
            panic!("wrong type");
        }
    } else {
        panic!("res should be \'EnumDef\'");
    };
}

#[test]
fn variables_test() {
    use tanit::{ast, ast::expressions};
    use tanit::{error_listener, lexer, parser};

    static SRC_PATH: &str = "./examples/values.tt";

    let lexer = lexer::Lexer::from_file(SRC_PATH, false);
    assert_eq!(lexer.is_ok(), true);

    let lexer = lexer.unwrap();

    let error_listener = error_listener::ErrorListener::new();

    let mut parser = parser::Parser::new(lexer, error_listener);

    let res = tanit::ast::functions::FunctionNode::parse_def(&mut parser).unwrap();

    let res = if let ast::Ast::FuncDef { node } = &res {
        assert_eq!(node.identifier, String::from("main"));
        assert_eq!(node.parameters.is_empty(), true);

        if let Type::Tuple { components } = &node.return_type {
            assert!(components.is_empty());
        } else {
            panic!("Type expected to be an empty tuple");
        }

        node.body.as_ref()
    } else {
        panic!("res has to be \'FuncDef\'");
    };

    let res = if let tanit::ast::Ast::Scope { node } = res.unwrap().as_ref() {
        &node.statements
    } else {
        panic!("res has to be \'LScope\'");
    };

    if let ast::Ast::VariableDef { node } = &res[0] {
        assert_eq!(node.identifier, "PI");
        assert!(!node.is_mutable);
        assert!(!node.is_global);
        assert!(matches!(node.var_type, Type::F32));
    } else {
        panic!("first statement has to be \'variable definition\'");
    }

    if let ast::Ast::Expression { node } = &res[1] {
        let (lhs, rhs) = if let ast::expressions::Expression::Binary {
            operation,
            lhs,
            rhs,
        } = node.as_ref()
        {
            assert_eq!(*operation, lexer::TokenType::Assign);
            (lhs.as_ref(), rhs.as_ref())
        } else {
            panic!("Expected binary expression");
        };

        assert!(matches!(lhs, ast::Ast::VariableDef { .. }));
        assert!(matches!(rhs, ast::Ast::Expression { .. }));
    } else {
        panic!("second statement has to be \'variable definition\'");
    }

    if let ast::Ast::Expression { node } = &res[2] {
        if let ast::expressions::Expression::Binary {
            operation,
            lhs,
            rhs,
        } = node.as_ref()
        {
            assert_eq!(*operation, lexer::TokenType::LShiftAssign);

            if let ast::Ast::Value { node } = lhs.as_ref() {
                match node {
                    ast::values::Value::Identifier(id) => {
                        assert_eq!(id, "radian");
                    }
                    _ => panic!("lhs has to be identifier"),
                }
            }

            let expr = if let tanit::ast::Ast::Expression { node } = rhs.as_ref() {
                node.as_ref()
            } else {
                panic!("rhs expected to be \'Expression\'");
            };

            if let expressions::Expression::Binary {
                operation,
                lhs,
                rhs,
            } = expr
            {
                assert_eq!(*operation, lexer::TokenType::Star);

                if let ast::Ast::Value { node } = lhs.as_ref() {
                    match node {
                        ast::values::Value::Integer(val) => {
                            assert_eq!(*val, 3);
                        }
                        _ => panic!("lhs has to be \'3\'"),
                    }
                } else {
                    panic!("lhs has to be \'Value\'");
                }

                let rhs = if let ast::Ast::Expression { node } = rhs.as_ref() {
                    node.as_ref()
                } else {
                    panic!("rhs has to be \'Expression\'");
                };

                if let expressions::Expression::Binary {
                    operation,
                    lhs,
                    rhs,
                } = rhs
                {
                    assert_eq!(*operation, tanit::lexer::TokenType::Slash);

                    if let tanit::ast::Ast::Value { node } = lhs.as_ref() {
                        match node {
                            tanit::ast::values::Value::Identifier(id) => {
                                assert_eq!(id, "PI");
                            }
                            _ => panic!("lhs has to be \'PI\'"),
                        }
                    }

                    if let tanit::ast::Ast::Value { node } = rhs.as_ref() {
                        match node {
                            tanit::ast::values::Value::Integer(val) => {
                                assert_eq!(*val, 4);
                            }
                            _ => panic!("rhs has to be \'4\'"),
                        }
                    }
                } else {
                    panic!("rhs has to be \'binary expression\'");
                }
            } else {
                panic!("Expected binary expression");
            }
        }
    } else {
        panic!("third statement has to be \'expression\'");
    }
}

#[test]
fn functions_test() {
    use tanit::ast;
    use tanit::{error_listener, lexer, parser};

    static SRC_PATH: &str = "./examples/functions.tt";

    let lexer = lexer::Lexer::from_file(SRC_PATH, false).unwrap();

    let error_listener = error_listener::ErrorListener::new();

    let mut parser = parser::Parser::new(lexer, error_listener);

    {
        let func = ast::functions::FunctionNode::parse_def(&mut parser).unwrap();

        let scope = if let ast::Ast::FuncDef { node } = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let ast::Ast::Scope { node } = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        assert!(matches!(node, ast::Ast::ReturnStmt { .. }));
    }

    {
        let func = ast::functions::FunctionNode::parse_def(&mut parser).unwrap();

        let scope = if let ast::Ast::FuncDef { node } = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let ast::Ast::Scope { node } = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        let (lhs, rhs) = if let ast::Ast::Expression { node } = node {
            if let ast::expressions::Expression::Binary {
                operation,
                lhs,
                rhs,
            } = node.as_ref()
            {
                assert_eq!(*operation, TokenType::Assign);
                (lhs.as_ref(), rhs.as_ref())
            } else {
                panic!("Expression expected to be binary");
            }
        } else {
            panic!("Expected expression");
        };

        assert!(matches!(lhs, ast::Ast::VariableDef { .. }));
        assert!(matches!(rhs, ast::Ast::Value { .. }));
    }
}

#[test]
fn types_test() {
    use tanit::ast;
    use tanit::{error_listener, lexer, parser};

    static SRC_PATH: &str = "./examples/types.tt";

    let lexer = lexer::Lexer::from_file(SRC_PATH, false).unwrap();

    let error_listener = error_listener::ErrorListener::new();

    let mut parser = parser::Parser::new(lexer, error_listener);

    let res = if let ast::Ast::FuncDef { node } =
        tanit::ast::functions::FunctionNode::parse_def(&mut parser).unwrap()
    {
        assert_eq!(node.identifier, String::from("main"));
        assert!(node.parameters.is_empty());

        if let Type::Tuple { components } = &node.return_type {
            assert!(components.is_empty());
        } else {
            panic!("Type expected to be an empty tuple");
        }

        node.body.unwrap()
    } else {
        panic!("res has to be \'function definition\'");
    };

    let statements = if let ast::Ast::Scope { node } = res.as_ref() {
        &node.statements
    } else {
        panic!("node has to be \'local scope\'");
    };

    if let ast::Ast::AliasDef { node } = &statements[0] {
        assert_eq!(node.identifier, "Items".to_string());

        if let Type::Template {
            identifier,
            arguments,
        } = &node.value
        {
            assert_eq!(identifier, "Vec");
            assert_eq!(arguments.len(), 1);
            if let Type::Custom(id) = &arguments[0] {
                assert_eq!(id, "Item");
            } else {
                panic!("Type is expected to be \"Item\"")
            }
        } else {
            panic!("Alias type expected to be an template type");
        }
    } else {
        panic!("res has to be \'alias definition\'");
    };
}
