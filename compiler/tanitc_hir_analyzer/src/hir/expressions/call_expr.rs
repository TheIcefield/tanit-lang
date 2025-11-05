use std::cmp::Ordering;

use tanitc_attributes::Mutability;
use tanitc_hir::hir::{
    expressions::call::{CallArg, CallExpr, NamedCallArg, PositionalCallArg},
    types::{FuncType, FuncTypeParam, Type},
};

use tanitc_lexer::location::Location;
use tanitc_messages::Message;

use crate::{symbol_table::type_info::TypeInfo, AnalyzeResult, Analyzer};

impl Analyzer {
    pub(crate) fn analyze_call_expr(&mut self, expr: &mut CallExpr) -> AnalyzeResult<()> {
        let expr_type = self.get_expr_type(&expr.expr);
        let Type::Func(func_type) = &expr_type.ty else {
            return Err(Message::new(
                expr.location,
                "Call something that is not a function",
            ));
        };

        self.check_args(expr, func_type)?;
        self.check_call_safety(expr, func_type)?;

        Ok(())
    }

    pub(crate) fn get_call_expr_type(&self, expr: &CallExpr) -> TypeInfo {
        let mut type_info = TypeInfo {
            ty: Type::new(),
            mutability: Mutability::Mutable,
            ..Default::default()
        };

        let expr_type = self.get_expr_type(&expr.expr);
        if matches!(expr_type.ty, Type::Func(_)) {
            type_info.ty = expr_type.ty;
        }

        type_info
    }

    fn check_args(&mut self, expr: &mut CallExpr, func_type: &FuncType) -> AnalyzeResult<()> {
        self.check_arg_count(func_type, &expr.arguments, expr.location)?;

        let mut positional_skipped = false;
        for call_arg in expr.arguments.iter_mut() {
            if let Err(err) = self.check_arg(func_type, call_arg, &mut positional_skipped) {
                self.error(err);
            }
        }

        Ok(())
    }

    fn check_positional_arg(
        &self,
        func_type: &FuncType,
        arg: &CallArg,
        positional_skipped: &mut bool,
    ) -> AnalyzeResult<usize> {
        let CallArg::Positional(PositionalCallArg {
            location,
            id: arg_idx,
            expr: arg_value,
        }) = arg
        else {
            return Err(Message::unreachable(
                arg.location(),
                format!("Expected CallArg::Position, actually: {arg:?}"),
            ));
        };

        if *positional_skipped {
            return Err(Message::from_string(
                *location,
                format!("Call: positional parameter \"{arg_idx}\" must be passed before notified",),
            ));
        }

        let Some(func_param) = func_type.parameters.get(*arg_idx) else {
            return Err(Message::from_string(
                *location,
                format!("Mismatched parameters: type \"{func_type}\" has no parameter {arg_idx}"),
            ));
        };

        let expr_type = self.get_expr_type(arg_value);
        if expr_type.ty != *func_param.ty {
            return Err(Message::from_string(
                *location,
                format!("Mismatched types. Call: positional parameter \"{arg_idx}\" has type \"{}\" but expected \"{}\"",
                    expr_type.ty, func_param.ty),
            ));
        }

        Ok(*arg_idx)
    }

    fn check_notified_arg(
        &self,
        func_type: &FuncType,
        arg: &CallArg,
        positional_skipped: &mut bool,
    ) -> AnalyzeResult<usize> {
        let CallArg::Notified(NamedCallArg {
            location,
            id: arg_id,
            expr: arg_value,
        }) = arg
        else {
            return Err(Message::unreachable(
                arg.location(),
                format!("Expected CallArg::Notified, actually: {arg:?}"),
            ));
        };

        *positional_skipped = true;

        // check if such parameter declared in the function
        for (
            param_index,
            FuncTypeParam {
                ty: param_type,
                id: param_name,
                ..
            },
        ) in func_type.parameters.iter().enumerate()
        {
            if *param_name == Some(*arg_id) {
                let arg_type = self.get_expr_type(arg_value);
                if **param_type != arg_type.ty {
                    return Err(Message::from_string(
                        *location,
                        format!("Mismatched types. Notified parameter \"{arg_id}\" has type \"{arg_type}\" but expected \"{param_type}\"", ),
                    ));
                }

                return Ok(param_index);
            }
        }

        Err(Message::from_string(
            *location,
            format!("No parameter named \"{arg_id}\" in function \"{func_type}\""),
        ))
    }

    fn check_arg(
        &mut self,
        func_type: &FuncType,
        arg: &mut CallArg,
        positional_skipped: &mut bool,
    ) -> AnalyzeResult<()> {
        let location = arg.location();

        let res = match arg {
            CallArg::Notified(_) => self.check_notified_arg(func_type, arg, positional_skipped),
            CallArg::Positional(_) => self.check_positional_arg(func_type, arg, positional_skipped),
        };

        match res {
            Ok(arg_position) => {
                let arg_value = match arg {
                    CallArg::Notified(NamedCallArg { expr, .. }) => Box::new(expr.clone()),
                    CallArg::Positional(PositionalCallArg { expr, .. }) => Box::new(expr.clone()),
                };

                *arg = CallArg::Positional(PositionalCallArg {
                    location,
                    id: arg_position,
                    expr: *arg_value,
                });
            }
            Err(err) => self.error(err),
        }

        Ok(())
    }

    fn check_arg_count(
        &self,
        func_type: &FuncType,
        arguments: &[CallArg],
        location: Location,
    ) -> AnalyzeResult<()> {
        let actual_len = arguments.len();
        let expected_len = func_type.parameters.len();

        let many_or_few = match actual_len.cmp(&expected_len) {
            Ordering::Greater => "many",
            Ordering::Less => "few",
            Ordering::Equal => "",
        };

        if actual_len != expected_len {
            return Err(Message::from_string(
                location,
                format!(
                    "Too {many_or_few} arguments passed in function, expected: {expected_len}, actually: {actual_len}",
                ),
            ));
        }

        Ok(())
    }

    fn check_call_safety(&mut self, expr: &CallExpr, func_type: &FuncType) -> AnalyzeResult<()> {
        if func_type.safety.is_unsafe() && self.get_current_safety().is_safe() {
            self.error(Message::new(
                expr.location,
                "Call unsafe function requires an unsafe function or block",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tanitc_attributes::{Mutability, Safety};
    use tanitc_hir::hir::{
        blocks::{Block, BlockAttributes},
        Hir,
    };
    use tanitc_hir_test::{
        create_block, create_call_expr, create_decimal_lit, create_func_def, create_main_func_def,
        create_var, create_var_def,
    };

    #[test]
    fn unsafe_call_bad_test() {
        // Given
        const FUNC_NAME: &str = "unsafe_func";
        let mut unsafe_func = create_func_def(FUNC_NAME, vec![], Type::unit(), vec![]);
        unsafe_func.attributes.safety = Safety::Unsafe;

        let main_func = create_main_func_def(vec![create_call_expr(FUNC_NAME, &[]).into()]);

        /*
         * unsafe func unsafe_func() { }
         * func main() {
         *     unsafe_func()
         * }
         */
        let mut program = Hir::from(Block {
            is_global: true,
            statements: vec![unsafe_func.into(), main_func.into()],
            ..Default::default()
        });

        // When
        let mut analyzer = Analyzer::new();
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str =
            "Semantic error: Call unsafe function requires an unsafe function or block";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn unsafe_call_good_test() {
        // Given
        const FUNC_NAME: &str = "unsafe_func";

        let mut unsafe_func = create_func_def(FUNC_NAME, vec![], Type::unit(), vec![]);
        unsafe_func.attributes.safety = Safety::Unsafe;

        let main_func = create_main_func_def(vec![Block {
            attributes: BlockAttributes {
                safety: Safety::Unsafe,
            },
            statements: vec![create_call_expr(FUNC_NAME, &[]).into()],
            ..Default::default()
        }
        .into()]);

        let mut program = Hir::from(create_block(vec![unsafe_func.into(), main_func.into()]));

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn good_func_call_by_ptr_test() {
        // Given
        const SOME_FUNC_NAME: &str = "some_func";
        let some_func = create_func_def(SOME_FUNC_NAME, vec![], Type::unit(), vec![]);

        const FUNC_PTR_NAME: &str = "func_ptr";
        let var_def = create_var_def(
            FUNC_PTR_NAME,
            Mutability::Immutable,
            Type::Auto,
            Some(create_var(SOME_FUNC_NAME)),
        );

        let call_expr = create_call_expr(FUNC_PTR_NAME, &[]);
        let main_func = create_main_func_def(vec![var_def.into(), call_expr.into()]);

        /*
         * func some_func() {}
         * func main() {
         *     var func_ptr = some_func
         *     func_ptr()
         * }
         */
        let mut program = Hir::from(create_block(vec![some_func.into(), main_func.into()]));

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn bad_func_call_by_ptr_test() {
        // Given
        const VAR_NAME: &str = "func_ptr";
        const VAR_VALUE: f64 = 4.14;

        let var_def = create_var_def(
            VAR_NAME,
            Mutability::Immutable,
            Type::Auto,
            Some(create_decimal_lit(VAR_VALUE)),
        );

        let call_expr = create_call_expr(VAR_NAME, &[]);
        let main_func = create_main_func_def(vec![var_def.into(), call_expr.into()]);

        /*
         * func main() {
         *     var a = 4.14
         *     a()
         * }
         */
        let mut program = Hir::from(create_block(vec![main_func.into()]));

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: Call something that is not a function";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }
}
