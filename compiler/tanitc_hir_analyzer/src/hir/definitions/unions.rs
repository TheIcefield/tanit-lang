use tanitc_hir::hir::{
    definitions::unions::UnionDef,
    expressions::{
        literal::{Literal, StructLiteral},
        Expression,
    },
};
use tanitc_ident::Ident;
use tanitc_messages::Message;

use std::collections::BTreeMap;

use crate::{
    symbol_table::entry::{Entry, StructFieldData, SymbolKind, UnionDefData},
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_union_def(&mut self, union_def: &mut UnionDef) -> AnalyzeResult<()> {
        if self.has_symbol(union_def.name.id) {
            return Err(Message::multiple_ids(union_def.location, union_def.name.id));
        }

        union_def.name.prefix = self.table.get_id();

        for internal in union_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        let mut fields = BTreeMap::<Ident, StructFieldData>::new();
        for (field_id, field_info) in union_def.fields.iter() {
            let Some(ty) = self.table.lookup_type(&field_info.ty.ty) else {
                self.error(Message::undefined_type(
                    field_info.ty.location,
                    field_info.ty.ty.to_string(),
                ));
                continue;
            };

            fields.insert(
                *field_id,
                StructFieldData {
                    struct_name: union_def.name,
                    ty: ty.ty,
                },
            );
        }

        self.add_symbol(Entry {
            name: union_def.name.id,
            is_static: true,
            kind: SymbolKind::from(UnionDefData {
                name: union_def.name,
                fields,
            }),
        });

        Ok(())
    }

    pub(crate) fn access_union_def(
        &mut self,
        union_data: &UnionDefData,
        node: &mut Expression,
    ) -> AnalyzeResult<()> {
        let Expression::Literal(Literal::Struct(StructLiteral {
            location,
            id: union_name,
            fields: value_comps,
        })) = node
        else {
            return Err(Message::unreachable(
                node.location(),
                format!("expected StructLiteral, actually: {}", node.kind_str()),
            ));
        };

        self.check_union_literal_components(value_comps, union_data, *location)?;

        *union_name = union_data.name;

        Ok(())
    }
}

/*
#[cfg(test)]
mod tests {
    use tanitc_attributes::{Mutability, Safety};
    use tanitc_hir::hir::{blocks::Block, expressions::Expression, Hir};
    use tanitc_hir_test::{get_func_def, get_union_def, get_var_def};
    use tanitc_ident::{Ident, Name};
    use tanitc_lexer::location::Location;
    use tanitc_ty::Type;

    use crate::Analyzer;

    const FIELD_NAME: &str = "field";


    fn get_access(var_name: &str) -> Expression {
        Expression {
            location: Location::default(),
            kind: ExpressionKind::Get {
                lhs: Box::new(Hir::from(Value {
                    kind: ValueKind::Identifier(Ident::from(var_name.to_string())),
                    location: Location::default(),
                })),
                rhs: Box::new(Hir::from(Value {
                    kind: ValueKind::Identifier(Ident::from(FIELD_NAME.to_string())),
                    location: Location::default(),
                })),
            },
        }
    }

    #[test]
    fn union_unsafe_access_good_test() {
        const UNION_NAME: &str = "UnionName";
        const FUNC_NAME: &str = "safe_func";
        const VAR_NAME: &str = "my_union";

        let mut func_def = get_func_def(
            FUNC_NAME,
            vec![
                get_var_def(
                    VAR_NAME,
                    Mutability::Immutable,
                    Type::Custom(Name::from(UNION_NAME.to_string())),
                    None,
                )
                .into(),
                get_access(VAR_NAME).into(),
            ],
        );
        func_def.attributes.safety = Safety::Unsafe;

        let mut program = Hir::from(Block {
            is_global: true,
            statements: vec![get_union_def(UNION_NAME, &[]).into(), func_def.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let messages = analyzer.messages_ref();
        if messages.has_errors() {
            panic!("{:?}", messages.errors_ref());
        }
    }

    #[test]
    fn union_safe_access_bad_test() {
        const UNION_NAME: &str = "UnionName";
        const FUNC_NAME: &str = "safe_func";
        const VAR_NAME: &str = "my_union";

        const EXPECTED_ERR: &str = "Semantic error: Access to union field is unsafe and requires an unsafe function or block";

        let func_def = get_func_def(
            FUNC_NAME,
            vec![
                get_var_def(
                    VAR_NAME,
                    Mutability::Immutable,
                    Type::Custom(Name::from(UNION_NAME.to_string())),
                    None,
                )
                .into(),
                get_access(VAR_NAME).into(),
            ],
        );

        let mut program = Hir::from(Block {
            is_global: true,
            statements: vec![get_union_def(UNION_NAME, &[]).into(), func_def.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let messages = analyzer.messages_ref();
        let errors = messages.errors_ref();

        assert!(!errors.is_empty());
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn union_work_test() {
        const SRC_TEXT: &str = "\nunion MyUnion\
                                \n{\
                                \n    f1: i32\
                                \n    f2: f32\
                                \n}\
                                \nfunc main() {\
                                \n    unsafe {\
                                \n        var s = MyUnion { \
                                \n                  f2: 2.0\
                                \n                }\
                                \n    }\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let mut hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        {
            let mut analyzer = Analyzer::new();
            analyzer.analyze_program(hir.as_mut()).unwrap();
        }

        {
            const HEADER_EXPECTED: &str = "typedef union {\
                                         \n    signed int f1;\
                                         \n    float f2;\
                                         \n} MyUnion;\
                                         \nvoid main();\n";

            const SOURCE_EXPECTED: &str = "void main()\
                                         \n{\
                                         \n    {\
                                         \n        MyUnion const s = (MyUnion)\
                                         \n        {\
                                         \n            .f2=2.0,\
                                         \n        };\
                                         \n    }\
                                         \n\
                                         \n}\n";

            let mut header_buffer = Vec::<u8>::new();
            let mut source_buffer = Vec::<u8>::new();
            let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

            writer.codegen_program(hir.as_ref()).unwrap();

            let header_res = String::from_utf8(header_buffer).unwrap();
            assert_str_eq!(HEADER_EXPECTED, header_res);

            let source_res = String::from_utf8(source_buffer).unwrap();
            assert_str_eq!(SOURCE_EXPECTED, source_res);
        }
    }

    #[test]
    fn union_in_module_work_test() {
        const SRC_TEXT: &str = "\nmodule mod {\
                                \n    union MyUnion {\
                                \n        x: i32\
                                \n        pub y: f32\
                                \n    }\
                                \n}\
                                \nfunc main() {\
                                \n    var u = mod::MyUnion { \
                                \n                  y: 2.0\
                                \n              }\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let mut hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        {
            let mut analyzer = Analyzer::new();
            analyzer.analyze_program(hir.as_mut()).unwrap();
        }

        {
            const HEADER_EXPECTED: &str = "typedef union {\
                                         \n    signed int x;\
                                         \n    float y;\
                                         \n} mod__MyUnion;\
                                         \nvoid main();\n";

            const SOURCE_EXPECTED: &str = "void main()\
                                         \n{\
                                         \n    mod__MyUnion const u = (mod__MyUnion)\
                                         \n    {\
                                         \n        .y=2.0,\
                                         \n    };\
                                         \n}\n";

            let mut header_buffer = Vec::<u8>::new();
            let mut source_buffer = Vec::<u8>::new();
            let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

            writer.codegen_program(hir.as_ref()).unwrap();

            let header_res = String::from_utf8(header_buffer).unwrap();
            assert_str_eq!(HEADER_EXPECTED, header_res);

            let source_res = String::from_utf8(source_buffer).unwrap();
            assert_str_eq!(SOURCE_EXPECTED, source_res);
        }
    }
}
*/
