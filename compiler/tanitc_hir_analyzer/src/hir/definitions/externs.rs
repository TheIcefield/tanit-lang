use tanitc_hir::{hir::definitions::externs::ExternDef, visitor::VisitorMut};

use crate::{AnalyzeResult, Analyzer};

impl Analyzer {
    pub(crate) fn analyze_extern_def(&mut self, extern_def: &mut ExternDef) -> AnalyzeResult<()> {
        for func_def in extern_def.functions.iter_mut() {
            if let Err(err) = self.visit_func_def(func_def) {
                self.error(err);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    /*
    #[test]
    fn extern_test() {
        const SRC_TEXT: &str = "\nextern \"C\" {\
                                \n    func hello(): i32
                                \n}\
                                \nfunc main() {\
                                \n    var res = hello()\
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
            const HEADER_EXPECTED: &str = "signed int hello();\
                                         \nvoid main();\n";

            const SOURCE_EXPECTED: &str = "void main()\
                                         \n{\
                                         \n    signed int const res = hello();\
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
