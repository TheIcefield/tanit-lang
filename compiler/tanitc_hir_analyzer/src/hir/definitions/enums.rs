use std::collections::BTreeMap;

use crate::{
    symbol_table::entry::{Entry, EnumData, EnumDefData, SymbolKind},
    AnalyzeResult, Analyzer,
};
use tanitc_hir::hir::{
    definitions::enums::EnumDef,
    expressions::{
        literal::{Integer, Literal},
        variable::Variable,
        Expression,
    },
};
use tanitc_ident::Ident;
use tanitc_messages::Message;

impl Analyzer {
    pub(crate) fn analyze_enum_def(&mut self, enum_def: &mut EnumDef) -> AnalyzeResult<()> {
        if self.has_symbol(enum_def.name.id) {
            return Err(Message::multiple_ids(enum_def.location, enum_def.name.id));
        }

        enum_def.name.prefix = self.table.get_id();

        let mut counter = 0usize;
        let mut enums = BTreeMap::<Ident, Entry>::new();
        for field in enum_def.units.iter_mut() {
            if let Some(value) = field.1 {
                counter = *value;
            }

            // mark unmarked enum fields
            *field.1 = Some(counter);

            enums.insert(
                *field.0,
                Entry {
                    name: *field.0,
                    is_static: true,
                    kind: SymbolKind::Enum(EnumData {
                        enum_name: enum_def.name,
                        value: counter,
                    }),
                },
            );

            counter += 1;
        }

        self.add_symbol(Entry {
            name: enum_def.name.id,
            is_static: true,
            kind: SymbolKind::from(EnumDefData {
                name: enum_def.name,
                enums,
            }),
        });

        Ok(())
    }

    pub(crate) fn access_enum(
        &self,
        enum_data: &EnumData,
        rhs: &mut Expression,
    ) -> AnalyzeResult<()> {
        let location = rhs.location();

        println!("FOR REPLACE: {rhs:#?}");

        *rhs = Expression::Variable(Variable {
            location,
            id: enum_data.enum_name,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_attributes::Mutability;
    use tanitc_hir::hir::types::Type;
    use tanitc_hir_test::{
        create_enum_def, create_main_func_def, create_program, create_scope_resolutions_expr,
        create_var_def,
    };

    use crate::Analyzer;

    #[test]
    fn enum_work_test() {
        // Given
        const ENUM_NAME: &str = "MyEnum";
        const FIRST_UNIT_NAME: &str = "First";
        const SECOND_UNIT_NAME: &str = "Second";
        const MAX_UNIT_NAME: &str = "Max";
        let enum_def = create_enum_def(
            ENUM_NAME,
            vec![
                (FIRST_UNIT_NAME, Some(1)),
                (SECOND_UNIT_NAME, None),
                (MAX_UNIT_NAME, None),
            ],
        );

        let var_value = Some(create_scope_resolutions_expr(&[
            ENUM_NAME,
            SECOND_UNIT_NAME,
        ]));
        let var_def = create_var_def("second", Mutability::Immutable, Type::Auto, var_value);

        let main_func = create_main_func_def(vec![var_def.into()]);

        /* enum MyEnum {
         *     One: 1
         *     Second
         *     Max
         * }
         *
         * func main() {
         *     var a = MyEnum::Second
         * };
         */
        let mut program = create_program(vec![enum_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        panic!("PROGRAM: {program:#?}");

        // Then
        res.expect("Expected no errors");
    }

    /*
        #[test]
        fn enum_in_module_work_test() {
            const SRC_TEXT: &str = "\nmodule color {\
                                    \n    enum Color {\
                                    \n        Red\
                                    \n        Green\
                                    \n        Blue\
                                    \n    }\
                                    \n}\
                                    \nfunc main() {\
                                    \n    var a = color::Color::Red\
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
                const HEADER_EXPECTED: &str = "typedef enum {\
                                             \n    Red = 0,\
                                             \n    Green = 1,\
                                             \n    Blue = 2,\
                                             \n} color__Color;\
                                             \nvoid main();\n";

                const SOURCE_EXPECTED: &str = "void main()\
                                             \n{\
                                             \n    color__Color const a = 0;\
                                             \n}\n";

                let mut header_buffer = Vec::<u8>::new();
                let mut source_buffer = Vec::<u8>::new();
                let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

                writer.codegen_program(hir.as_ref()).unwrap();

                let mut res = String::from_utf8(header_buffer).unwrap();
                assert_str_eq!(HEADER_EXPECTED, res);

                res = String::from_utf8(source_buffer).unwrap();
                assert_str_eq!(SOURCE_EXPECTED, res);
            }
        }
    */
}
