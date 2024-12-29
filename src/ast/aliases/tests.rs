use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::Parser;

use std::str::FromStr;

#[test]
fn alias_test() {
    use crate::ast::functions::FunctionDef;
    use crate::parser::lexer::Lexer;

    static SRC_PATH: &str = "./examples/types.tt";

    let lexer = Lexer::from_file(SRC_PATH, false).unwrap();

    let mut parser = Parser::new(lexer);

    let res = if let Ast::FuncDef { node } = FunctionDef::parse(&mut parser).unwrap() {
        assert!(node.identifier == Identifier::from_str("main").unwrap());
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

    let statements = if let Ast::Scope { node } = res.as_ref() {
        &node.statements
    } else {
        panic!("node has to be \'local scope\'");
    };

    if let Ast::AliasDef { node } = &statements[0] {
        assert!(node.identifier == Identifier::from_str("Items").unwrap());

        if let Type::Template {
            identifier,
            arguments,
        } = &node.value
        {
            assert!(*identifier == Identifier::from_str("Vec").unwrap());
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
