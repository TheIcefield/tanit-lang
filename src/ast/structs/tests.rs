use super::StructDef;
use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::{lexer::Lexer, Parser};
use std::str::FromStr;

#[test]
fn struct_test() {
    static SRC_PATH: &str = "./examples/structs.tt";

    let lexer = Lexer::from_file(SRC_PATH, false).unwrap();

    let mut parser = Parser::new(lexer);

    let res = StructDef::parse(&mut parser).unwrap();

    if let Ast::StructDef { node } = res {
        assert!(node.identifier == Identifier::from_str("S1").unwrap());

        let field_type = node
            .fields
            .get(&Identifier::from_str("f1").unwrap())
            .unwrap();
        assert!(matches!(field_type, Type::I32));

        let field_type = node
            .fields
            .get(&Identifier::from_str("f2").unwrap())
            .unwrap();

        if let Type::Template {
            identifier,
            arguments,
        } = &field_type
        {
            let expected_id = Identifier::from_str("Vec").unwrap();

            assert!(*identifier == expected_id);
            assert_eq!(arguments.len(), 1);
            assert_eq!(arguments[0], Type::I32);
        } else {
            panic!("wrong type");
        }
    } else {
        panic!("res should be \'StructDef\'");
    };
}
