use tanit::ast::types::Type;

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

    let res = tanit::ast::modules::parse(&mut parser);

    assert_eq!(res.is_some(), true);

    let res = res.unwrap();

    let res = if let ast::Ast::ModuleDef { node } = res {
        assert_eq!(node.identifier, String::from("M1"));

        node.body.statements[0].clone()
    } else {
        panic!("res should be \'ModuleDef\'");
    };

    if let ast::Ast::ModuleDef { node } = res {
        assert_eq!(node.identifier, String::from("M2"));
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

    let res = tanit::ast::structs::parse_struct_def(&mut parser);

    assert_eq!(res.is_some(), true);

    let res = res.unwrap();

    if let ast::Ast::StructDef { node } = res {
        assert_eq!(node.identifier, String::from("S1"));

        assert_eq!(node.fields[0].identifier, String::from("f1"));
        assert_eq!(node.fields[0].is_field, true);
        assert_eq!(node.fields[0].is_global, false);
        assert_eq!(node.fields[0].is_mutable, true);

        let mut field_type = node.fields[0].var_type.clone();

        assert!(matches!(field_type, Type::I32));

        assert_eq!(node.fields[1].identifier, String::from("f2"));

        field_type = node.fields[1].var_type.clone();

        if let Type::Template {
            identifier,
            arguments,
        } = &field_type
        {
            assert_eq!(*identifier, String::from("Vec"));
            assert_eq!(arguments.len(), 1);
            assert!(matches!(arguments[0], Type::I32));
        } else {
            panic!("wrong type");
        }
    } else {
        panic!("res should be \'ModuleDef\'");
    };
}

#[test]
fn variables_test() {
    use tanit::ast;
    use tanit::{error_listener, lexer, parser};

    static SRC_PATH: &str = "./examples/values.tt";

    let lexer = lexer::Lexer::from_file(SRC_PATH, false);
    assert_eq!(lexer.is_ok(), true);

    let lexer = lexer.unwrap();

    let error_listener = error_listener::ErrorListener::new();

    let mut parser = parser::Parser::new(lexer, error_listener);

    let res = tanit::ast::functions::parse_func_def(&mut parser);
    assert_eq!(res.is_some(), true);

    let res = res.unwrap();

    let res = if let ast::Ast::FuncDef { node } = res {
        assert_eq!(node.identifier, String::from("main"));
        assert_eq!(node.parameters.is_empty(), true);
        assert_eq!(node.is_static, false);

        if let Type::Tuple { components } = &node.return_type {
            assert!(components.is_empty());
        } else {
            panic!("Type expected to be an empty tuple");
        }

        node.body
    } else {
        panic!("res has to be \'Scope\'");
    };

    let mut res = res.unwrap();

    if let tanit::ast::Ast::VariableDef { node } = res.statements.remove(0) {
        assert_eq!(node.identifier, String::from("PI"));
        assert!(!node.is_mutable);
        assert!(!node.is_global);
        assert!(!node.is_field);
        assert!(matches!(node.var_type, Type::F32));
    } else {
        panic!("first statement has to be \'variable definition\'");
    }

    if let tanit::ast::Ast::Expression { node } = res.statements.remove(0) {
        assert_eq!(node.operation.unwrap(), tanit::lexer::TokenType::Assign);

        if let tanit::ast::Ast::Value { node } = *node.lhs.unwrap() {
            match node {
                tanit::ast::values::ValueType::Identifier(id) => {
                    assert_eq!(id, "PI".to_string());
                }
                _ => panic!("lhs has to be identifier"),
            }
        }

        if let tanit::ast::Ast::Value { node } = *node.rhs.unwrap() {
            match node {
                tanit::ast::values::ValueType::Integer(val) => {
                    assert_eq!(val, 2);
                }
                _ => panic!("rhs has to be \'2\'"),
            }
        }
    } else {
        panic!("second statement has to be \'variable definition\'");
    }

    if let tanit::ast::Ast::Expression { node } = res.statements.remove(0) {
        assert_eq!(
            node.operation.unwrap(),
            tanit::lexer::TokenType::LShiftAssign
        );

        if let tanit::ast::Ast::Value { node } = *node.lhs.unwrap() {
            match node {
                tanit::ast::values::ValueType::Identifier(id) => {
                    assert_eq!(id, "radian".to_string());
                }
                _ => panic!("lhs has to be identifier"),
            }
        }

        if let tanit::ast::Ast::Expression { node } = *node.rhs.unwrap() {
            assert_eq!(node.operation.unwrap(), tanit::lexer::TokenType::Star);

            if let tanit::ast::Ast::Value { node } = *node.lhs.unwrap() {
                match node {
                    tanit::ast::values::ValueType::Integer(val) => {
                        assert_eq!(val, 3);
                    }
                    _ => panic!("lhs has to be \'3\'"),
                }
            }

            if let tanit::ast::Ast::Expression { node } = *node.rhs.unwrap() {
                assert_eq!(node.operation.unwrap(), tanit::lexer::TokenType::Slash);

                if let tanit::ast::Ast::Value { node } = *node.lhs.unwrap() {
                    match node {
                        tanit::ast::values::ValueType::Identifier(id) => {
                            assert_eq!(id, "PI".to_string());
                        }
                        _ => panic!("lhs has to be \'PI\'"),
                    }
                }

                if let tanit::ast::Ast::Value { node } = *node.rhs.unwrap() {
                    match node {
                        tanit::ast::values::ValueType::Integer(val) => {
                            assert_eq!(val, 4);
                        }
                        _ => panic!("rhs has to be \'4\'"),
                    }
                }
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

    if let ast::Ast::FuncDef { mut node } =
        tanit::ast::functions::parse_func_def(&mut parser).unwrap()
    {
        assert_eq!(node.identifier, String::from("f"));
        assert!(!node.is_static);
        assert!(matches!(node.return_type, Type::F32));

        let arg = node.parameters.remove(0);
        assert_eq!(arg.identifier, "a".to_string());
        assert!(matches!(arg.var_type, Type::F32));

        let arg = node.parameters.remove(0);
        assert_eq!(arg.identifier, "b".to_string());
        assert!(matches!(arg.var_type, Type::F32));

        assert_eq!(node.body.is_some(), true);
    } else {
        panic!("res has to be \'function definition\'");
    };

    let res = if let ast::Ast::FuncDef { node } =
        tanit::ast::functions::parse_func_def(&mut parser).unwrap()
    {
        assert_eq!(node.identifier, String::from("main"));
        assert!(!node.is_static);
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

    let res = if let tanit::ast::Ast::Expression { node } = &res.statements[0] {
        assert_eq!(
            node.operation.clone().unwrap(),
            tanit::lexer::TokenType::Assign
        );

        let res = if let tanit::ast::Ast::Value { node } = *node.rhs.clone().unwrap() {
            node
        } else {
            panic!("res has to be \'expression\'")
        };

        let res = match res {
            tanit::ast::values::ValueType::Call(node) => {
                assert_eq!(node.identifier, "f".to_string());
                assert_eq!(node.arguments.len(), 2);
                node.arguments
            }
            _ => panic!("value has to be \'call\'"),
        };

        res
    } else {
        panic!("res has to be \'expression\'");
    };

    if let tanit::ast::Ast::Value { node } = &res[0] {
        match node {
            tanit::ast::values::ValueType::Identifier(id) => {
                assert_eq!(*id, "a".to_string());
            }
            _ => panic!("first arg has to be \'identifier\'"),
        }
    } else {
        panic!("first arg has to be \'value\'");
    }

    if let tanit::ast::Ast::Value { node } = &res[1] {
        match node {
            tanit::ast::values::ValueType::Integer(val) => {
                assert_eq!(*val, 1);
            }
            _ => panic!("second arg has to be \'1\'"),
        }
    } else {
        panic!("second arg has to be \'value\'");
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
        tanit::ast::functions::parse_func_def(&mut parser).unwrap()
    {
        assert_eq!(node.identifier, String::from("main"));
        assert!(!node.is_static);
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

    if let tanit::ast::Ast::AliasDef { node } = &res.statements[0] {
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

    // if let tanit::ast::Ast::AliasDef { node } = &res.statements[0] {
    //     assert_eq!(node.identifier, "Items".to_string());

    //     if let Type::Template { identifier, arguments } = &node.value {
    //         assert_eq!(identifier, "Vec");
    //         assert_eq!(arguments.len(), 1);
    //         if let Type::Custom(id) = &arguments[0] {
    //             assert_eq!(id, "Item");
    //         } else {
    //             panic!("Type is expected to be \"Item\"")
    //         }
    //     } else {
    //         panic!("Alias type expected to be an template type");
    //     }
    // } else {
    //     panic!("res has to be \'alias definition\'");
    // };
}
