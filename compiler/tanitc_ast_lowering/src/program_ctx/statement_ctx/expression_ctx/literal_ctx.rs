use tanitc_ast::program_ctx::statement_ctx::expression_ctx::literal_ctx::{
    array_literal_ctx::ArrayLiteralCtx, struct_literal_ctx::StructLiteralCtx,
    tuple_literal_ctx::TupleLiteralCtx, LiteralCtx,
};
use tanitc_hir::hir::expressions::{
    literal::{ArrayLiteral, Decimal, Integer, Literal, StructLiteral, Text, TupleLiteral},
    Expression,
};
use tanitc_ident::Ident;
use tanitc_lexer::token::{lexeme::Lexeme, Token};
use tanitc_messages::Message;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_literal_ctx(&mut self, ctx: &LiteralCtx) -> AstLowResult<Literal> {
        match ctx {
            LiteralCtx::Text(tkn) => self.low_text_literal_ctx(tkn).map(Literal::Text),
            LiteralCtx::Integer(tkn) => self.low_integer_literal_ctx(tkn).map(Literal::Integer),
            LiteralCtx::Decimal(tkn) => self.low_decimal_literal_ctx(tkn).map(Literal::Decimal),
            LiteralCtx::Array(ctx) => self.low_array_literal_ctx(ctx).map(Literal::Array),
            LiteralCtx::Tuple(ctx) => self.low_tuple_literal_ctx(ctx).map(Literal::Tuple),
            LiteralCtx::Struct(ctx) => self.low_struct_literal_ctx(ctx).map(Literal::Struct),
        }
    }

    fn low_integer_literal_ctx(&self, token: &Token) -> AstLowResult<Integer> {
        let location = token.get_location();

        let Lexeme::Integer(value_str) = token.lexeme_ref() else {
            unreachable!()
        };

        let value = value_str
            .parse::<usize>()
            .map_err(|err| Message::new(location, err))?;

        Ok(Integer { location, value })
    }

    fn low_decimal_literal_ctx(&self, token: &Token) -> AstLowResult<Decimal> {
        let location = token.get_location();

        let Lexeme::Decimal(value_str) = token.lexeme_ref() else {
            unreachable!()
        };

        let value = value_str
            .parse::<f64>()
            .map_err(|err| Message::new(location, err))?;

        Ok(Decimal { location, value })
    }

    fn low_text_literal_ctx(&self, token: &Token) -> AstLowResult<Text> {
        let location = token.get_location();

        let Lexeme::Text(value) = token.lexeme_ref() else {
            unreachable!()
        };

        Ok(Text {
            location,
            value: value.to_string(),
        })
    }

    fn low_array_literal_ctx(&mut self, ctx: &ArrayLiteralCtx) -> AstLowResult<ArrayLiteral> {
        let location = ctx.lsb_tkn.get_location();
        let mut elements = Vec::<Expression>::new();

        for (expr_ctx, _) in ctx.elements.iter() {
            let Some(expr_ctx) = expr_ctx else {
                continue;
            };

            match self.low_expression_ctx(expr_ctx) {
                Err(err) => self.error(err),
                Ok(expr) => elements.push(expr),
            }
        }

        Ok(ArrayLiteral { location, elements })
    }

    fn low_tuple_literal_ctx(&mut self, ctx: &TupleLiteralCtx) -> AstLowResult<TupleLiteral> {
        let location = ctx.lparen_tkn.get_location();
        let mut units = Vec::<Expression>::new();

        for (expr_ctx, _) in ctx.elements.iter() {
            let Some(expr_ctx) = expr_ctx else {
                continue;
            };

            match self.low_expression_ctx(expr_ctx) {
                Err(err) => self.error(err),
                Ok(expr) => units.push(expr),
            }
        }

        Ok(TupleLiteral { location, units })
    }

    fn low_struct_literal_ctx(&mut self, ctx: &StructLiteralCtx) -> AstLowResult<StructLiteral> {
        let name = self.low_name_spec_ctx(&ctx.name_ctx)?;
        let location = name.location;
        let mut fields = Vec::<(Ident, Expression)>::new();

        for (expr_ctx, _) in ctx.elements.iter() {
            let Some(expr_ctx) = expr_ctx else {
                continue;
            };

            let id = expr_ctx.name_ctx.identifier();

            match self.low_expression_ctx(&expr_ctx.expression_ctx) {
                Err(err) => self.error(err),
                Ok(expr) => fields.push((id, expr)),
            }
        }

        Ok(StructLiteral {
            location,
            name,
            fields,
        })
    }
}
