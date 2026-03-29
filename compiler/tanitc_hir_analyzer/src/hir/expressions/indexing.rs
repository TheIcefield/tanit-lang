use tanitc_hir::hir::{
    expressions::{indexing::IndexingExpr, variable::Variable, Expression},
    type_spec::Type,
};
use tanitc_messages::Message;

use crate::{
    symbol_table::{entry::SymbolKind, type_info::TypeInfo},
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_indexing_expr(&mut self, expr: &mut IndexingExpr) -> AnalyzeResult<()> {
        let location = expr.location;

        self.analyze_expression(expr.lhs.as_mut())?;
        self.analyze_expression(expr.index.as_mut())?;

        match expr.lhs.as_ref() {
            Expression::Variable(Variable { name: var_name, .. }) => {
                let var_entry = self
                    .table
                    .lookup_name_spec(var_name)
                    .map_err(|err| Message::new(location, err))?;

                let SymbolKind::VarDef(var_data) = &var_entry.kind else {
                    return Err(Message::new(
                        location,
                        format!("{var_name} is not an variable"),
                    ));
                };

                let Type::Array { .. } = &var_data.var_type else {
                    return Err(Message::new(
                        location,
                        format!("{var_name} is not an array"),
                    ));
                };
            }
            _ => {
                return Err(Message::new(
                    location,
                    format!("Can't index {}", expr.lhs.kind_str()),
                ));
            }
        }

        let index_ty = self.get_expr_type(&expr.index);
        if !index_ty.ty.is_integer() {
            return Err(Message::new(
                expr.index.location(),
                format!("Invalid index type: {}", index_ty.ty),
            ));
        }

        Ok(())
    }

    pub(crate) fn get_indexing_expr_type(&self, expr: &IndexingExpr) -> TypeInfo {
        let mut lhs_type = self.get_expr_type(&expr.lhs);
        let Type::Array { ref value_type, .. } = &lhs_type.ty else {
            unreachable!()
        };

        lhs_type.ty = value_type.as_ref().clone();
        lhs_type
    }
}

#[cfg(test)]
mod tests {
    /*

    #[test]
    fn array_work_test() {
        const SRC_TEXT: &str = "\nfunc main() {\
                                \n    var mut arr_1: [f32: 6]\
                                \n    var arr_2: [i32: 3] = [4, 5, 6]\
                                \n    var arr_3 = [1.0, 2.0, 3.0]\
                                \n    arr_1[1 + 1] = 7.0\
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
            const HEADER_EXPECTED: &str = "void main();\n";

            const SOURCE_EXPECTED: &str = "void main()\
                                         \n{\
                                         \n    float arr_1[6];\
                                         \n    signed int const arr_2[3] = { 4, 5, 6 };\
                                         \n    float const arr_3[3] = { 1.0, 2.0, 3.0 };\
                                         \n    arr_1[1 + 1] = 7.0;\
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
    fn immutable_array_bad_test() {
        const SRC_TEXT: &str = "\nfunc main() {\
                                \n    var arr = [1.0, 2.0, 3.0] # immutable\
                                \n    arr[0] = 7.0\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let mut hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        {
            const EXPECTED_ERR: &str = "Semantic error: Cannot mutate immutable object of type \"f32\" is immutable in current scope";

            let mut analyzer = Analyzer::new();
            let messages = analyzer.analyze_program(hir.as_mut()).err().unwrap();
            let errors = messages.errors_ref();

            assert_eq!(errors.len(), 1);
            assert_str_eq!(errors[0].text, EXPECTED_ERR);
        }
    }

    #[test]
    fn strange_index_array_bad_test() {
        const SRC_TEXT: &str = "\nfunc main() {\
                                \n    var mut arr = [1.0, 2.0, 3.0]\
                                \n    arr[3.14] = 7.0\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let mut hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        {
            const EXPECTED_ERR: &str = "Semantic error: Invalid index type: f32";

            let mut analyzer = Analyzer::new();
            let messages = analyzer.analyze_program(hir.as_mut()).err().unwrap();
            let errors = messages.errors_ref();

            assert_eq!(errors.len(), 1);
            assert_str_eq!(errors[0].text, EXPECTED_ERR);
        }
    }
     */
}
