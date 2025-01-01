use super::{VariantDef, VariantField};
use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::{lexer::Lexer, Parser};

use std::str::FromStr;

#[test]
fn variant_test() {
    static SRC_TEXT: &str = "variant V1\
                             {\n
                                 f1\n
                                 f2(i32, i32)\n
                                 f3 {\n
                                     f1: i32\n
                                     f2: f32\n
                                 }\n
                             }";

    let lexer = Lexer::from_text(SRC_TEXT, false).unwrap();

    let mut parser = Parser::new(lexer);

    let res = VariantDef::parse(&mut parser).unwrap();

    if let Ast::VariantDef { node } = &res {
        assert!(node.identifier == Identifier::from_str("V1").unwrap());

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
}
