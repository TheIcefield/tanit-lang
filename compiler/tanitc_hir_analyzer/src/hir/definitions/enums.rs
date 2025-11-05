use std::collections::BTreeMap;

use crate::{
    symbol_table::entry::{Entry, EnumData, EnumDefData, SymbolKind},
    AnalyzeResult, Analyzer,
};
use tanitc_hir::hir::{
    definitions::enums::EnumDef,
    expressions::{
        literal::{Integer, Literal},
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
        for field in enum_def.fields.iter_mut() {
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

        *rhs = Expression::Literal(Literal::Integer(Integer {
            location,
            value: enum_data.value,
        }));

        Ok(())
    }
}

/*
#[cfg(test)]
mod tests {
    #[test]
    fn enum_work_test() {
        const SRC_TEXT: &str = "\npub enum MyEnum {\
                                \n    One: 1\
                                \n    Second\
                                \n    Max\
                                \n}\
                                \nfunc main() {\
                                \n    var a = MyEnum::Second\
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
                                         \n    One = 1,\
                                         \n    Second = 2,\
                                         \n    Max = 3,\
                                         \n} MyEnum;\
                                         \nvoid main();\n";

            const SOURCE_EXPECTED: &str = "void main()\
                                         \n{\
                                         \n    MyEnum const a = 2;\
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
}
*/
