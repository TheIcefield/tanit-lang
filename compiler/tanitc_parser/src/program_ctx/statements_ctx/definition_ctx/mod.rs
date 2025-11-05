use tanitc_ast::program_ctx::statement_ctx::definition_ctx::DefinitionCtx;
use tanitc_lexer::token::lexeme::Lexeme;
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

pub(crate) mod alias_def_ctx;
pub(crate) mod const_def_ctx;
pub(crate) mod enum_def_ctx;
pub(crate) mod func_def_ctx;
pub(crate) mod impl_ctx;
pub(crate) mod module_def_ctx;
pub(crate) mod static_def_ctx;
pub(crate) mod struct_def_ctx;
pub(crate) mod union_def_ctx;
pub(crate) mod var_def_ctx;
pub(crate) mod variant_def_ctx;

impl Parser {
    pub fn parse_definition_ctx(&mut self) -> ParseResult<DefinitionCtx> {
        let next = self.peek_token().ok_or(Message::reached_eof())?;
        match next.lexeme_ref() {
            Lexeme::KwAlias => self.parse_alias_def_ctx().map(DefinitionCtx::Alias),
            Lexeme::KwEnum => self.parse_enum_def_ctx().map(DefinitionCtx::Enum),
            Lexeme::KwStruct => self.parse_struct_def_ctx().map(DefinitionCtx::Struct),
            Lexeme::KwUnion => self.parse_union_def_ctx().map(DefinitionCtx::Union),
            Lexeme::KwFunc => self.parse_func_def_ctx().map(DefinitionCtx::Func),
            Lexeme::KwVariant => self.parse_variant_def_ctx().map(DefinitionCtx::Variant),
            Lexeme::KwStatic => self.parse_static_def_ctx().map(DefinitionCtx::Static),
            Lexeme::KwConst => self.parse_const_def_ctx().map(DefinitionCtx::Const),
            Lexeme::KwVar => self.parse_var_def_ctx().map(DefinitionCtx::Variable),
            Lexeme::KwImpl => self.parse_impl_ctx().map(DefinitionCtx::Impl),
            Lexeme::KwExtern => self.parse_extern_ctx().map(DefinitionCtx::Extern),
            Lexeme::KwDef | Lexeme::KwModule => {
                self.parse_module_def_ctx().map(DefinitionCtx::Module)
            }
            _ => Err(Message::unexpected_token(&next, &[])),
        }
    }
}
