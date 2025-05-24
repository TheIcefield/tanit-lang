use std::collections::BTreeMap;

use pretty_assertions::assert_str_eq;
use tanitc_ast::{Ast, Block, FunctionDef, StructDef, TypeSpec};
use tanitc_ident::Ident;
use tanitc_serializer::ron_writer::RonWriter;
use tanitc_ty::Type;

#[test]
fn ast_write_test() {
    let ast = Ast::from(Block {
        statements: vec![
            Ast::from(StructDef {
                identifier: Ident::from("MyStruct".to_string()),
                fields: {
                    let mut fields = BTreeMap::new();
                    fields.insert(
                        Ident::from("a".to_string()),
                        TypeSpec {
                            ty: Type::I16,
                            ..Default::default()
                        },
                    );
                    fields.insert(
                        Ident::from("b".to_string()),
                        TypeSpec {
                            ty: Type::I16,
                            ..Default::default()
                        },
                    );
                    fields
                },
                ..Default::default()
            }),
            Ast::from(FunctionDef {
                identifier: Ident::from(format!("MyFunction")),
                ..Default::default()
            }),
        ],
        ..Default::default()
    });

    let expected = if let Ast::Block(block) = &ast {
        format!("{block:#?}")
    } else {
        unreachable!()
    };

    let mut buffer = Vec::<u8>::new();
    let mut writer = RonWriter::new(&mut buffer).unwrap();
    ast.accept(&mut writer).unwrap();

    let res = String::from_utf8(buffer).unwrap();
    assert_str_eq!(expected, res);
}
