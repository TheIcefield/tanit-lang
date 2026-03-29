use tanitc_attributes::{Mutability, Safety};
use tanitc_hir::hir::{
    expressions::{
        unary::{UnaryExpr, UnaryOperation},
        Expression,
    },
    type_spec::{RefType, Type},
};
use tanitc_messages::Message;

use crate::{
    symbol_table::{entry::SymbolKind, type_info::TypeInfo},
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_unary_expr(&mut self, expr: &mut UnaryExpr) -> AnalyzeResult<()> {
        let location = expr.location;

        self.analyze_expression(&mut expr.node)?;
        let node_type = self.get_expr_type(&expr.node);

        let does_mutate = expr.operation == UnaryOperation::RefMut;

        if let Expression::Variable(var) = expr.node.as_ref() {
            let entry = self
                .table
                .lookup_name_spec(&var.name)
                .map_err(|err| Message::new(location, err))?;

            if let SymbolKind::VarDef(var_data) = &entry.kind {
                if var_data.mutability.is_const() && does_mutate {
                    return Err(Message::new(
                        location,
                        format!("Mutable reference to immutable variable \"{}\"", var.name),
                    ));
                }
            }
        }

        if node_type.ty.is_pointer()
            && UnaryOperation::Deref == expr.operation
            && self.get_current_safety() != Safety::Unsafe
        {
            return Err(Message::new(
                location,
                "Dereferencing raw pointer require unsafe function or block",
            ));
        }

        Ok(())
    }

    pub(crate) fn get_unary_expr_type(&self, expr: &UnaryExpr) -> TypeInfo {
        let node_type = self.get_expr_type(&expr.node);

        let (is_ref, mutability) = match &expr.operation {
            UnaryOperation::Ref => (true, Mutability::Immutable),
            UnaryOperation::RefMut => (true, Mutability::Mutable),
            _ => (false, Mutability::Immutable),
        };

        if is_ref {
            return TypeInfo {
                ty: Type::Ref(RefType {
                    ref_to: Box::new(node_type.ty.clone()),
                    mutability,
                }),
                mutability,
                members: node_type.members,
                ..Default::default()
            };
        }

        node_type
    }
}

/*
#[cfg(test)]
mod tests {
    use tanitc_attributes::{Mutability, Safety};
    use tanitc_hir::hir::{
        blocks::{Block, BlockAttributes},
        definitions::variables::VariableDef,
        expressions::Expression,
        Hir,
    };
    use tanitc_hir_test::get_func_def;
    use tanitc_ident::Ident;
    use tanitc_lexer::location::Location;
    use tanitc_ty::{PtrType, Type};

    use crate::Analyzer;

    /* Creates:
     * var var_name: i32 = 0
     */
    fn get_var_init(var_name: &str) -> Expression {
        Expression {
            location: Location::default(),
            kind: ExpressionKind::Binary {
                operation: BinaryOperation::Assign,
                lhs: Box::new(
                    VariableDef {
                        identifier: Ident::from(var_name.to_string()),
                        var_type: Type::I32,
                        ..Default::default()
                    }
                    .into(),
                ),
                rhs: Box::new(
                    Value {
                        location: Location::default(),
                        kind: ValueKind::Integer(0),
                    }
                    .into(),
                ),
            },
        }
    }

    /* Creates:
     * var ptr_name: *i32 = &var_name
     */
    fn get_raw_ptr_init(ptr_name: &str, var_name: &str) -> Expression {
        Expression {
            location: Location::default(),
            kind: ExpressionKind::Binary {
                operation: BinaryOperation::Assign,
                lhs: Box::new(
                    VariableDef {
                        identifier: Ident::from(ptr_name.to_string()),
                        var_type: Type::Ptr(PtrType {
                            ptr_to: Box::new(Type::I32),
                            mutability: Mutability::Immutable,
                        }),
                        ..Default::default()
                    }
                    .into(),
                ),
                rhs: Box::new(
                    Expression {
                        location: Location::default(),
                        kind: ExpressionKind::Unary {
                            operation: UnaryOperation::Ref,
                            node: Box::new(
                                Value {
                                    location: Location::default(),
                                    kind: ValueKind::Identifier(Ident::from(var_name.to_string())),
                                }
                                .into(),
                            ),
                        },
                    }
                    .into(),
                ),
            },
        }
    }

    /* Creates:
     * *ptr_name
     */
    fn get_raw_ptr_deref(ptr_name: &str) -> Expression {
        Expression {
            location: Location::default(),
            kind: ExpressionKind::Unary {
                operation: UnaryOperation::Deref,
                node: Box::new(
                    Value {
                        location: Location::default(),
                        kind: ValueKind::Identifier(Ident::from(ptr_name.to_string())),
                    }
                    .into(),
                ),
            },
        }
    }

    #[test]
    fn check_deref_ptr_safety_good_test() {
        const MAIN_FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "my_var";
        const PTR_NAME: &str = "my_ptr";

        /*
         * func main() {
         *    var my_var: i32 = 0
         *    var my_ptr: *i32 = &my_var
         *    unsafe { *my_ptr }
         * }
         */
        let mut program = Hir::from(Block {
            is_global: true,
            statements: vec![get_func_def(
                MAIN_FUNC_NAME,
                vec![
                    get_var_init(VAR_NAME).into(),
                    get_raw_ptr_init(PTR_NAME, VAR_NAME).into(),
                    Block {
                        is_global: false,
                        attributes: BlockAttributes {
                            safety: Safety::Unsafe,
                        },
                        statements: vec![get_raw_ptr_deref(PTR_NAME).into()],
                        ..Default::default()
                    }
                    .into(),
                ],
            )
            .into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let messages = analyzer.messages_ref();
        if messages.has_errors() {
            panic!("{:#?}", messages.errors_ref());
        }
    }

    #[test]
    fn check_deref_ptr_safety_bad_test() {
        const MAIN_FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "my_var";
        const PTR_NAME: &str = "my_ptr";
        const EXPECTED_ERR: &str =
            "Semantic error: Dereferencing raw pointer require unsafe function or block";

        let mut program = Hir::from(Block {
            is_global: true,
            statements: vec![get_func_def(
                MAIN_FUNC_NAME,
                vec![
                    get_var_init(VAR_NAME).into(),
                    get_raw_ptr_init(PTR_NAME, VAR_NAME).into(),
                    get_raw_ptr_deref(PTR_NAME).into(),
                ],
            )
            .into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let messages = analyzer.messages_ref();
        let errors = messages.errors_ref();

        assert!(!errors.is_empty());
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    /*

    #[test]
    fn immutable_deref_test() {
        const SRC_TEXT: &str = "\nfunc main() {\
                                \n    var value = 50\
                                \n    var ref = &value\
                                \n    if 1 {\
                                \n        *ref = 10\
                                \n    }\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let mut hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        {
            const EXPECTED_ERROR_TEXT: &str =
                "Semantic error: Reference \"ref\" is immutable in current scope";

            let mut analyzer = Analyzer::new();
            let messages = analyzer.analyze_program(hir.as_mut()).err().unwrap();
            let errors = messages.errors_ref();

            assert_str_eq!(
                errors.first().expect("Expected errors").text,
                EXPECTED_ERROR_TEXT
            );
        }
    }
     */

    /*

    #[test]
    fn mutable_deref_test() {
        const SRC_TEXT: &str = "\nfunc main() {\
                                \n    var mut value = 50\
                                \n    var ref = &mut value\
                                \n    if 1 {\
                                \n        *ref = 10\
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
            const HEADER_EXPECTED: &str = "void main();\n";
            const SOURCE_EXPECTED: &str = "void main()\
                                         \n{\
                                         \n    signed int value = 50;\
                                         \n    signed int * const ref = &value;\
                                         \n    if (1)\
                                         \n    {\
                                         \n        *ref = 10;\
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
     */

    /*

    #[test]
    fn mutable_ref_to_immutable_var_test() {
        const SRC_TEXT: &str = "\nfunc main() {\
                                \n    var value = 50\
                                \n    var ref = &mut value\
                                \n    if 1 {\
                                \n        *ref = 10\
                                \n    }\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let mut hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        {
            const EXPECTED_ERROR_TEXT: &str =
                "Semantic error: Mutable reference to immutable variable \"value\"";

            let mut analyzer = Analyzer::new();
            let mesages = analyzer.analyze_program(hir.as_mut()).err().unwrap();
            let errors = mesages.errors_ref();

            assert_str_eq!(
                errors.first().expect("Expected errors").text,
                EXPECTED_ERROR_TEXT
            );
        }
    }
     */

    /*

    #[test]
    fn immutable_deref_param_test() {
        const SRC_TEXT: &str = "\nfunc bar(p: &i32) {\
                                \n    *p = 10 # expected error\
                                \n}\
                                \nfunc main() {\
                                \n    var value = 50\
                                \n    bar(&value)\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let mut hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        {
            const EXPECTED_ERROR_TEXT: &str =
                "Semantic error: Reference \"p\" is immutable in current scope";

            let mut analyzer = Analyzer::new();
            let messages = analyzer.analyze_program(hir.as_mut()).err().unwrap();
            let errors = messages.errors_ref();

            assert_str_eq!(
                errors.first().expect("Expected errors").text,
                EXPECTED_ERROR_TEXT
            );
        }
    }
      */

    /*

    #[test]
    fn mutable_deref_param_test() {
        const SRC_TEXT: &str = "\nfunc bar(p: &mut i32) {\
                                \n    *p = 10\
                                \n}\
                                \nfunc main() {\
                                \n    var mut value = 50\
                                \n    bar(&mut value)\
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
            const HEADER_EXPECTED: &str = "void bar(signed int * const p);\
                                         \nvoid main();\n";
            const SOURCE_EXPECTED: &str = "void bar(signed int * const p)\
                                         \n{\
                                         \n    *p = 10;\
                                         \n}\
                                         \nvoid main()\
                                         \n{\
                                         \n    signed int value = 50;\
                                         \n    bar(&value);\
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
    */

}
*/
