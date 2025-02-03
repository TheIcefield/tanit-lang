use super::{VariantDef, VariantField};
use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::{lexer::Lexer, Parser};
use crate::serializer::XmlWriter;

use std::str::FromStr;

#[test]
fn variant_def_test() {
    const SRC_TEXT: &str = "\nvariant MyVariant\
                            \n{\
                            \n    f1\
                            \n    f2(i32, i32)\
                            \n    f3 {\
                            \n        f1: i32\
                            \n        f2: f32\
                            \n    }\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let variant_node = VariantDef::parse(&mut parser).unwrap();

    if let Ast::VariantDef(node) = &variant_node {
        assert!(node.identifier == Identifier::from_str("MyVariant").unwrap());

        assert!(matches!(
            node.fields.get(&Identifier::from_str("f1").unwrap()),
            Some(&VariantField::Common)
        ));

        if let VariantField::TupleLike(components) = node
            .fields
            .get(&Identifier::from_str("f2").unwrap())
            .unwrap()
        {
            assert_eq!(components.len(), 2);
            assert_eq!(components[0], Type::I32);
            assert_eq!(components[1], Type::I32);
        } else {
            panic!("wrong type");
        }

        let field = node
            .fields
            .get(&Identifier::from_str("f3").unwrap())
            .unwrap();
        if let VariantField::StructLike(components) = &field {
            assert_eq!(components.len(), 2);
            assert!(matches!(
                components.get(&Identifier::from_str("f1").unwrap()),
                Some(&Type::I32)
            ));
            assert!(matches!(
                components.get(&Identifier::from_str("f2").unwrap()),
                Some(&Type::F32)
            ));
        } else {
            panic!("wrong type");
        }
    } else {
        panic!("res should be \'VariantDef\'");
    };

    {
        const EXPECTED: &str = "\n<variant-definition name=\"MyVariant\">\
                                \n    <field name=\"f1\"/>\
                                \n    <field name=\"f2\">\
                                \n        <type style=\"primitive\" name=\"i32\"/>\
                                \n        <type style=\"primitive\" name=\"i32\"/>\
                                \n    </field>\
                                \n    <field name=\"f3\">\
                                \n        <field name=\"f1\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </field>\
                                \n        <field name=\"f2\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </field>\
                                \n    </field>\
                                \n</variant-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        variant_node.serialize(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }
}
