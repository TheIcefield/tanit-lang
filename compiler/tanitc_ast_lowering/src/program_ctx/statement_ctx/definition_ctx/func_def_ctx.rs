use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx,
    definition_ctx::func_def_ctx::{
        FuncDefCommonParamCtx, FuncDefCtx, FuncDefParamCtx, FuncDefParamKindCtx, FuncDefParamsCtx,
        FuncDefSelfRefParamCtx, FuncDefSelfValParamCtx,
    },
};

use tanitc_attributes::{Mutability, Visibility};
use tanitc_hir::hir::{
    definitions::{
        functions::{FunctionAttributes, FunctionDef, FunctionParam},
        variables::{VariableAttributes, VariableDef},
    },
    types::Type,
};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_func_def_ctx(
        &mut self,
        func_def_ctx: &FuncDefCtx,
    ) -> AstLowResult<FunctionDef> {
        let location = func_def_ctx.func_tkn.get_location();
        let attributes = self.low_func_def_attributes(&func_def_ctx.attributes_ctx)?;
        let name = self.low_name_ctx(&func_def_ctx.name_ctx);
        let parameters = self.low_func_def_params_ctx(&func_def_ctx.params_ctx, name.id)?;

        let return_type = if let Some(type_ctx) = &func_def_ctx.return_type_ctx {
            self.low_type_ctx(&type_ctx.type_ctx)?.ty
        } else {
            Type::unit()
        };

        let body = if let Some(body_ctx) = &func_def_ctx.body_ctx {
            Some(Box::new(self.low_block_ctx(body_ctx)?))
        } else {
            None
        };

        Ok(FunctionDef {
            location,
            attributes,
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn low_func_def_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<FunctionAttributes> {
        Ok(FunctionAttributes {
            publicity: self.low_publicity_token(&ctx.pub_tkn),
            safety: self.low_safety(&ctx.safe_tkn, &ctx.unsafe_tkn)?,
        })
    }

    fn low_func_def_params_ctx(
        &mut self,
        params_ctx: &FuncDefParamsCtx,
        func_id: Ident,
    ) -> AstLowResult<Vec<FunctionParam>> {
        let mut params = Vec::<FunctionParam>::new();

        for (idx, param_ctx) in params_ctx.params_ctx.iter().enumerate() {
            match self.low_func_def_param_ctx(param_ctx, idx) {
                Ok(param) => params.push(param),
                Err(msg) => self.error(msg.map_in_func_def(func_id)),
            }
        }

        Ok(params)
    }

    fn low_func_def_param_ctx(
        &mut self,
        param_ctx: &FuncDefParamCtx,
        idx: usize,
    ) -> AstLowResult<FunctionParam> {
        match &param_ctx.param_ctx {
            FuncDefParamKindCtx::CommonParam(param_ctx) => {
                self.low_func_def_common_param_ctx(param_ctx)
            }
            FuncDefParamKindCtx::SelfRef(param_ctx) => {
                self.low_func_def_self_ref_param_ctx(param_ctx, idx)
            }
            FuncDefParamKindCtx::SelfVal(param_ctx) => {
                self.low_func_def_self_val_param_ctx(param_ctx, idx)
            }
        }
    }

    fn low_func_def_self_ref_param_ctx(
        &mut self,
        param_ctx: &FuncDefSelfRefParamCtx,
        idx: usize,
    ) -> AstLowResult<FunctionParam> {
        if idx > 0 {
            return Err(self.wrong_self_param_position(param_ctx.self_tkn.get_location()));
        }

        Ok(FunctionParam::SelfRef(if param_ctx.mut_tkn.is_some() {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        }))
    }

    fn low_func_def_self_val_param_ctx(
        &mut self,
        param_ctx: &FuncDefSelfValParamCtx,
        idx: usize,
    ) -> AstLowResult<FunctionParam> {
        if idx > 0 {
            return Err(self.wrong_self_param_position(param_ctx.self_tkn.get_location()));
        }

        Ok(FunctionParam::SelfVal(if param_ctx.mut_tkn.is_some() {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        }))
    }

    fn low_func_def_common_param_ctx(
        &mut self,
        param_ctx: &FuncDefCommonParamCtx,
    ) -> AstLowResult<FunctionParam> {
        let location = param_ctx.name_ctx.name_tkn.get_location();
        let attributes = VariableAttributes::default();
        let identifier = self.low_name_ctx(&param_ctx.name_ctx).id;
        let var_type = self.low_type_ctx(&param_ctx.type_ctx)?.ty;
        let visibility = Visibility::Local;
        let mutability = self.low_mut_token(&param_ctx.mut_tkn);

        Ok(FunctionParam::Common(VariableDef {
            location,
            attributes,
            identifier,
            var_type,
            visibility,
            mutability,
            value: None,
        }))
    }

    fn wrong_self_param_position(&mut self, location: Location) -> Message {
        Message::new(location, "unexpected \"self\" parameter in function. Must be the first parameter of an associated function")
    }
}
