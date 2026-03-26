use tanitc_hir::hir::expressions::{
    binary::{BinaryExpr, BinaryOperation},
    call::CallArg,
    literal::Literal,
    unary::UnaryOperation,
    Expression,
};

use crate::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_expression(&mut self, expr: &Expression) -> std::io::Result<()> {
        use std::io::Write;

        let old_mode = self.mode;
        self.mode = CodeGenMode::SourceOnly;

        match expr {
            Expression::Unary(expr) => {
                match &expr.operation {
                    UnaryOperation::Add => write!(self, "+")?,
                    UnaryOperation::Sub => write!(self, "-")?,
                    UnaryOperation::RefMut | UnaryOperation::Ref => write!(self, "&")?,
                    UnaryOperation::Not => write!(self, "~")?,
                    UnaryOperation::Deref => write!(self, "*")?,
                };

                self.generate_expression(expr.node.as_ref())?;
            }
            Expression::Binary(BinaryExpr {
                operation,
                lhs,
                rhs,
                ..
            }) => {
                self.generate_expression(lhs)?;

                if BinaryOperation::ScopeRes == *operation {
                    write!(self, "__")?;
                } else {
                    write!(self, " {operation} ")?;
                }

                self.generate_expression(rhs)?;
            }
            Expression::Conversion(conversion) => {
                write!(self, "(({})", conversion.ty.get_c_type())?;
                self.generate_expression(&conversion.expr)?;
                write!(self, ")")?;
            }
            Expression::Indexing(expr) => {
                self.generate_expression(&expr.lhs)?;

                write!(self, "[")?;
                self.generate_expression(&expr.index)?;
                write!(self, "]")?;
            }
            Expression::Call(call) => {
                self.generate_expression(&call.expr)?;

                write!(self, "(")?;

                /* at this point, all arguments must be converted to positional */
                if let Some(first) = call.arguments.first() {
                    self.generate_call_param(first)?;
                }

                for arg in call.arguments.iter().skip(1) {
                    write!(self, ", ")?;
                    self.generate_call_param(arg)?;
                }

                write!(self, ")")?;
            }
            Expression::Literal(lit) => self.generate_literal(lit)?,
            Expression::Variable(var) => write!(self, "{}", var.id)?,
        }

        self.mode = old_mode;
        Ok(())
    }

    fn generate_literal(&mut self, literal: &Literal) -> std::io::Result<()> {
        use std::io::Write;

        match literal {
            Literal::Integer(val) => write!(self, "{}", val.value)?,
            Literal::Decimal(val) => write!(self, "{:?}", val.value)?,
            Literal::Text(val) => write!(self, "\"{}\"", val.value)?,
            Literal::Struct(struct_lit) => {
                // create anonimous variable
                write!(self, "({})", struct_lit.id)?;

                if struct_lit.fields.is_empty() {
                    write!(self, " {{ }}")?;
                } else {
                    let fields_count = struct_lit.fields.len();
                    let indentation = self.indentation();
                    self.indent += 1;

                    writeln!(self, "\n{indentation}{{")?;
                    for (i, (field_name, field_val)) in struct_lit.fields.iter().enumerate() {
                        write!(self, "{indentation}    .{field_name}=")?;
                        self.generate_expression(field_val)?;

                        if i < fields_count {
                            writeln!(self, ",")?;
                        }
                    }

                    self.indent -= 1;
                    write!(self, "{indentation}}}")?;
                }
            }
            Literal::Array(arr_lit) => {
                let arr_len = arr_lit.elements.len();

                write!(self, "{{ ")?;

                for (el_idx, el) in arr_lit.elements.iter().enumerate() {
                    self.generate_expression(el)?;

                    if el_idx != arr_len - 1 {
                        write!(self, ", ")?;
                    }
                }

                write!(self, " }}")?;
            }
            Literal::Tuple(tuple_lit) => {
                let units_count = tuple_lit.units.len();

                write!(self, "{{ ")?;

                for (unit_idx, unit) in tuple_lit.units.iter().enumerate() {
                    self.generate_expression(unit)?;

                    if unit_idx != units_count - 1 {
                        write!(self, ", ")?;
                    }
                }

                write!(self, " }}")?;
            }
        }

        Ok(())
    }

    fn generate_call_param(&mut self, arg: &CallArg) -> std::io::Result<()> {
        match arg {
            CallArg::Positional(arg) => self.generate_expression(&arg.expr),
            CallArg::Notified(arg) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Notified argument \"{}\" must be eliminated at this point",
                    arg.id
                ),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tanitc_hir::hir::{blocks::Block, types::Type, Hir};
    use tanitc_hir_test::{
        create_array_lit, create_call_expr, create_decimal_lit, create_func_def,
        create_integer_lit, create_struct_lit, create_text_lit, create_tuple_lit, create_var,
    };
    use tanitc_options::CompileOptions;

    use pretty_assertions::assert_str_eq;

    #[test]
    fn codegen_values_test() {
        // Given
        const CRATE_NAME: &str = "my_crate";
        const FUNC_NAME: &str = "just_func";

        let program = Hir::from(Block {
            is_global: true,
            statements: vec![create_func_def(
                FUNC_NAME,
                vec![],
                Type::unit(),
                vec![
                    create_text_lit("text").into(),
                    create_var("var_name").into(),
                    create_call_expr("empty_func_name", vec![]).into(),
                    create_call_expr("func_with_1p", vec![create_decimal_lit(0.0)]).into(),
                    create_call_expr(
                        "func_with_2p",
                        vec![create_decimal_lit(0.0), create_decimal_lit(2.0)],
                    )
                    .into(),
                    create_struct_lit("MyEmptyStruct", &[]).into(),
                    create_struct_lit("StructWith1F", &[("f1", create_decimal_lit(1.1))]).into(),
                    create_struct_lit(
                        "StructWith2F",
                        &[
                            ("f1", create_integer_lit(0)),
                            ("f2", create_decimal_lit(2.2)),
                        ],
                    )
                    .into(),
                    create_array_lit(vec![]).into(),
                    create_array_lit(vec![create_integer_lit(0)]).into(),
                    create_array_lit(vec![create_integer_lit(1), create_integer_lit(2)]).into(),
                    create_tuple_lit(vec![]).into(),
                    create_tuple_lit(vec![create_integer_lit(0)]).into(),
                    create_tuple_lit(vec![create_integer_lit(1), create_integer_lit(2)]).into(),
                ],
            )
            .into()],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::with_compile_options(
            &mut header_buffer,
            &mut source_buffer,
            CompileOptions {
                crate_name: CRATE_NAME.to_string(),
                ..Default::default()
            },
        );

        // When
        writer.codegen_program(&program).unwrap();

        // Then
        const HEADER_EXPECTED: &str = "void just_func();\n";
        const SOURCE_EXPECTED: &str = "#include \"my_crate.tt.h\"\
                                     \n\
                                     \nvoid just_func()\
                                     \n{\
                                     \n    \"text\";\
                                     \n    var_name;\
                                     \n    empty_func_name();\
                                     \n    func_with_1p(0.0);\
                                     \n    func_with_2p(0.0, 2.0);\
                                     \n    (MyEmptyStruct) { };\
                                     \n    (StructWith1F)\
                                     \n    {\
                                     \n        .f1=1.1,\
                                     \n    };\
                                     \n    (StructWith2F)\
                                     \n    {\
                                     \n        .f1=0,\
                                     \n        .f2=2.2,\
                                     \n    };\
                                     \n    {  };\
                                     \n    { 0 };\
                                     \n    { 1, 2 };\
                                     \n    {  };\
                                     \n    { 0 };\
                                     \n    { 1, 2 };\
                                     \n}\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
