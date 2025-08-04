use tanitc_ast::ast::{
    blocks::Block,
    functions::FunctionDef,
    structs::{StructDef, StructFieldInfo},
    types::TypeSpec,
    Ast,
};
use tanitc_ident::Ident;
use tanitc_ty::Type;

use pretty_assertions::assert_str_eq;

use std::collections::BTreeMap;

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
                        StructFieldInfo {
                            ty: TypeSpec {
                                ty: Type::I16,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    );
                    fields.insert(
                        Ident::from("b".to_string()),
                        StructFieldInfo {
                            ty: TypeSpec {
                                ty: Type::I16,
                                ..Default::default()
                            },
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

    const EXPECTED: &str = "Block {\
                          \n    location: Location {\
                          \n        row: 1,\
                          \n        col: 1,\
                          \n    },\
                          \n    attributes: BlockAttributes {\
                          \n        safety: Inherited,\
                          \n    },\
                          \n    statements: [\
                          \n        StructDef {\
                          \n            location: Location {\
                          \n                row: 1,\
                          \n                col: 1,\
                          \n            },\
                          \n            attributes: StructAttributes {\
                          \n                publicity: Private,\
                          \n            },\
                          \n            identifier: MyStruct,\
                          \n            fields: {\
                          \n                a: StructFieldInfo {\
                          \n                    ty: TypeSpec {\
                          \n                        location: Location {\
                          \n                            row: 1,\
                          \n                            col: 1,\
                          \n                        },\
                          \n                        info: ParsedTypeInfo {\
                          \n                            mutability: Immutable,\
                          \n                        },\
                          \n                        ty: i16,\
                          \n                    },\
                          \n                    attributes: StructFieldAttributes {\
                          \n                        publicity: Private,\
                          \n                    },\
                          \n                },\
                          \n                b: StructFieldInfo {\
                          \n                    ty: TypeSpec {\
                          \n                        location: Location {\
                          \n                            row: 1,\
                          \n                            col: 1,\
                          \n                        },\
                          \n                        info: ParsedTypeInfo {\
                          \n                            mutability: Immutable,\
                          \n                        },\
                          \n                        ty: i16,\
                          \n                    },\
                          \n                    attributes: StructFieldAttributes {\
                          \n                        publicity: Private,\
                          \n                    },\
                          \n                },\
                          \n            },\
                          \n            internals: [],\
                          \n        },\
                          \n        FunctionDef {\
                          \n            location: Location {\
                          \n                row: 1,\
                          \n                col: 1,\
                          \n            },\
                          \n            attributes: FunctionAttributes {\
                          \n                publicity: Private,\
                          \n                safety: Inherited,\
                          \n            },\
                          \n            identifier: MyFunction,\
                          \n            return_type: TypeSpec {\
                          \n                location: Location {\
                          \n                    row: 1,\
                          \n                    col: 1,\
                          \n                },\
                          \n                info: ParsedTypeInfo {\
                          \n                    mutability: Immutable,\
                          \n                },\
                          \n                ty: ( ),\
                          \n            },\
                          \n            parameters: [],\
                          \n            body: None,\
                          \n        },\
                          \n    ],\
                          \n    is_global: false,\
                          \n}";

    let res = format!("{ast:#?}");
    assert_str_eq!(EXPECTED, res);
}
