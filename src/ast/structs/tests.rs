use super::StructDef;
use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::{lexer::Lexer, Parser};
use crate::serializer::XmlWriter;
use std::str::FromStr;

#[test]
fn struct_def_test() {
    const SRC_TEXT: &str = "\nstruct MyStruct\
                            \n{\
                            \n    f1: i32\
                            \n    f2: Vec<i32>\
                            \n}";

    let mut parser =
        Parser::new(Lexer::from_text(SRC_TEXT, false).expect("Failed to create lexer"));

    let struct_node = StructDef::parse(&mut parser).unwrap();

    if let Ast::StructDef { node } = &struct_node {
        assert!(node.identifier == Identifier::from_str("MyStruct").unwrap());

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

    {
        const EXPECTED: &str = "\n<struct-definition name=\"MyStruct\">\
                                \n    <field name=\"f1\">\
                                \n        <type style=\"primitive\" name=\"i32\"/>\
                                \n    </field>\
                                \n    <field name=\"f2\">\
                                \n        <type style=\"generic\" name=\"Vec\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </type>\
                                \n    </field>\
                                \n</struct-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        struct_node.serialize(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }
}
