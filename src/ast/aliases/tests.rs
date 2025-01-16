use crate::ast::{aliases::AliasDef, identifiers::Identifier, types::Type, Ast};
use crate::parser::{lexer::Lexer, Parser};
use crate::serializer::XmlWriter;

use std::str::FromStr;

#[test]
fn alias_def_test() {
    static SRC_TEXT: &str = "alias MyAlias = f32";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT, false).expect("Lexer creation failed"));

    let alias_node = AliasDef::parse(&mut parser).unwrap();

    {
        const EXPECTED: &str = "\n<alias-definition name=\"MyAlias\">\
                                \n    <type style=\"primitive\" name=\"f32\"/>\
                                \n</alias-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        alias_node.serialize(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }
}

#[test]
fn alias_in_func_test() {
    use crate::ast::functions::FunctionDef;

    static SRC_TEXT: &str = "func main() -> ()\
                            {\
                                alias Items = Vec<Item>\
                            }";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT, false).expect("Lexer creation failed"));

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
