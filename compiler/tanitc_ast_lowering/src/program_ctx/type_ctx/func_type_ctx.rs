use tanitc_ast::program_ctx::type_ctx::func_type_ctx::{
    FuncTypeCtx, FuncTypeParamCtx, FuncTypeParamsCtx,
};
use tanitc_attributes::Safety;
use tanitc_hir::hir::type_spec::{FuncType, FuncTypeParam, Type, TypeSpec};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_func_type_ctx(&self, type_ctx: &FuncTypeCtx) -> AstLowResult<TypeSpec> {
        let location = type_ctx.func_tkn.get_location();

        let parameters = self.low_func_type_params_ctx(&type_ctx.params_ctx)?;
        let return_type = Box::new(if let Some(return_type_ctx) = &type_ctx.return_type {
            self.low_type_ctx(&return_type_ctx.type_ctx)?.ty
        } else {
            Type::unit()
        });

        let ty = Type::Func(FuncType {
            parameters,
            return_type,
            safety: Safety::Safe,
        });

        Ok(TypeSpec { location, ty })
    }

    fn low_func_type_params_ctx(
        &self,
        params_ctx: &FuncTypeParamsCtx,
    ) -> AstLowResult<Vec<FuncTypeParam>> {
        let mut params = Vec::<FuncTypeParam>::new();

        for param in params_ctx.parameters.iter() {
            params.push(self.low_func_type_param_ctx(param)?)
        }

        Ok(params)
    }

    fn low_func_type_param_ctx(&self, param_ctx: &FuncTypeParamCtx) -> AstLowResult<FuncTypeParam> {
        Ok(FuncTypeParam {
            ty: Box::new(self.low_type_ctx(&param_ctx.type_ctx)?.ty),
            id: None,
        })
    }
}
