use tanitc_ast::{Ast, VariantField};
use tanitc_ident::Ident;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::XmlWriter;
use tanitc_ty::Type;

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

    let variand_id = Ident::from("MyVariant".to_string());
    let f1_id = Ident::from("f1".to_string());
    let f2_id = Ident::from("f2".to_string());
    let f3_id = Ident::from("f3".to_string());

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let variant_node = parser.parse_variant_def().unwrap();

    if let Ast::VariantDef(node) = &variant_node {
        assert!(node.identifier == variand_id);

        assert!(matches!(
            node.fields.get(&f1_id),
            Some(&VariantField::Common)
        ));

        if let VariantField::TupleLike(components) = node.fields.get(&f2_id).unwrap() {
            assert_eq!(components.len(), 2);
            assert_eq!(components[0].get_type(), Type::I32);
            assert_eq!(components[1].get_type(), Type::I32);
        } else {
            panic!("wrong type");
        }

        let field = node.fields.get(&f3_id).unwrap();
        if let VariantField::StructLike(components) = &field {
            assert_eq!(components.len(), 2);
            assert!(matches!(
                components.get(&f1_id).map(|val| { val.get_type() }),
                Some(Type::I32)
            ));
            assert!(matches!(
                components.get(&f2_id).map(|val| { val.get_type() }),
                Some(Type::F32)
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

        variant_node.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }
}
