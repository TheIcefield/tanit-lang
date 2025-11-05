use tanitc_ast::program_ctx::statement_ctx::expression_ctx::{
    binary_ctx::{BinaryCtx, BinaryOpCtx},
    conversion_ctx::ConversionCtx,
    literal_ctx::{tuple_literal_ctx::TupleLiteralCtx, LiteralCtx},
    paren_ctx::ParenCtx,
    unary_ctx::{UnaryCtx, UnaryOpCtx},
    ExpressionCtx,
};
use tanitc_lexer::token::lexeme::Lexeme;
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

pub(crate) mod call_ctx;
pub(crate) mod indexing_ctx;
pub(crate) mod literal_ctx;

impl Parser {
    pub fn parse_expression_ctx(&mut self) -> Result<ExpressionCtx, Message> {
        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(false);

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let expr = match next.lexeme_ref() {
            Lexeme::Plus
            | Lexeme::Minus
            | Lexeme::Ampersand
            | Lexeme::Star
            | Lexeme::Not
            | Lexeme::LParen => self.parse_factor()?,
            _ => self.parse_assign()?,
        };

        self.set_ignore_nl_option(old_opt);

        Ok(expr)
    }

    pub fn parse_factor(&mut self) -> ParseResult<ExpressionCtx> {
        let next = self.peek_token().ok_or(Message::reached_eof())?;

        match next.lexeme_ref() {
            lexem if *lexem == Lexeme::Ampersand => Ok(ExpressionCtx::Unary(UnaryCtx {
                unary_op_ctx: UnaryOpCtx::Ref(
                    self.consume_token(lexem.clone())?,
                    self.consume_token(Lexeme::KwMut).ok(),
                ),
                expression_ctx: Box::new(self.parse_expression_ctx()?),
            })),

            lexem if *lexem == Lexeme::Plus => Ok(ExpressionCtx::Unary(UnaryCtx {
                unary_op_ctx: UnaryOpCtx::Add(self.consume_token(lexem.clone())?),
                expression_ctx: Box::new(self.parse_expression_ctx()?),
            })),

            lexem if *lexem == Lexeme::Minus => Ok(ExpressionCtx::Unary(UnaryCtx {
                unary_op_ctx: UnaryOpCtx::Sub(self.consume_token(lexem.clone())?),
                expression_ctx: Box::new(self.parse_expression_ctx()?),
            })),

            lexem if lexem.is_integer() || lexem.is_integer() || *lexem == Lexeme::Lsb => {
                self.parse_literal_ctx().map(ExpressionCtx::Literal)
            }

            lexem if lexem.is_identifier() => {
                let identifier = self.consume_identifier()?;

                let old_opt = self.does_ignore_nl();
                self.set_ignore_nl_option(true);

                let expression_ctx = if self.is_next(Lexeme::LParen) {
                    self.parse_call_ctx(Box::new(ExpressionCtx::Variable(identifier)))
                        .map(ExpressionCtx::Call)
                } else if self.is_next(Lexeme::Lcb) {
                    self.parse_struct_literal_ctx(identifier)
                        .map(LiteralCtx::Struct)
                        .map(ExpressionCtx::Literal)
                } else if self.is_next(Lexeme::Lsb) {
                    self.parse_indexing_ctx(Box::new(ExpressionCtx::Variable(identifier)))
                        .map(ExpressionCtx::Indexing)
                } else {
                    Ok(ExpressionCtx::Variable(identifier))
                };

                self.set_ignore_nl_option(old_opt);
                expression_ctx
            }

            Lexeme::LParen => self.parse_paren(),

            _ => Err(Message::from_string(
                next.get_location(),
                format!("Unexpected token \"{next}\" within expression"),
            )),
        }
    }

    fn parse_paren(&mut self) -> ParseResult<ExpressionCtx> {
        let lparen_tkn = self.consume_token(Lexeme::LParen)?;

        // If parsed `()` then we return empty tuple
        if self.is_next(Lexeme::RParen) {
            return Ok(ExpressionCtx::Literal(LiteralCtx::Tuple(TupleLiteralCtx {
                lparen_tkn,
                elements: Vec::new(),
                rparen_tkn: self.consume_token(Lexeme::RParen)?,
            })));
        }

        let first_expr = self.parse_expression_ctx()?;

        // If parsed one expression, we return expression
        if self.is_next(Lexeme::RParen) {
            return Ok(ExpressionCtx::ParenCtx(ParenCtx {
                lparen_tkn,
                expression_ctx: Box::new(first_expr),
                rparen_tkn: self.consume_token(Lexeme::RParen)?,
            }));
        }

        // It's tuple
        Ok(ExpressionCtx::Literal(LiteralCtx::Tuple(TupleLiteralCtx {
            lparen_tkn,
            elements: self.parse_tuple_literal_elements_ctx()?,
            rparen_tkn: self.consume_token(Lexeme::RParen)?,
        })))
    }
}

impl Parser {
    fn parse_assign(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_logical_or()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        let binary_op_ctx = match next.lexeme_ref() {
            op if *op == Lexeme::Assign => BinaryOpCtx::Assign(self.consume_token(op.clone())?),
            op if *op == Lexeme::AddAssign => {
                BinaryOpCtx::AddAssign(self.consume_token(op.clone())?)
            }
            op if *op == Lexeme::SubAssign => {
                BinaryOpCtx::SubAssign(self.consume_token(op.clone())?)
            }
            op if *op == Lexeme::MulAssign => {
                BinaryOpCtx::MulAssign(self.consume_token(op.clone())?)
            }
            op if *op == Lexeme::DivAssign => {
                BinaryOpCtx::DivAssign(self.consume_token(op.clone())?)
            }
            op if *op == Lexeme::ModAssign => {
                BinaryOpCtx::ModAssign(self.consume_token(op.clone())?)
            }
            op if *op == Lexeme::OrAssign => {
                BinaryOpCtx::BitOrAssign(self.consume_token(op.clone())?)
            }
            op if *op == Lexeme::AndAssign => {
                BinaryOpCtx::BitAndAssign(self.consume_token(op.clone())?)
            }
            op if *op == Lexeme::XorAssign => {
                BinaryOpCtx::BitXorAssign(self.consume_token(op.clone())?)
            }
            op if *op == Lexeme::LShiftAssign => {
                BinaryOpCtx::LeftShiftAssign(self.consume_token(op.clone())?)
            }
            op if *op == Lexeme::RShiftAssign => {
                BinaryOpCtx::RightShiftAssign(self.consume_token(op.clone())?)
            }
            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }

    fn parse_logical_or(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_logical_and()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        let binary_op_ctx = match next.lexeme_ref() {
            exp if *exp == Lexeme::Or => BinaryOpCtx::LogicOr(self.consume_token(exp.clone())?),

            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }

    fn parse_logical_and(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_bitwise_or()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        let binary_op_ctx = match next.lexeme_ref() {
            exp if *exp == Lexeme::And => BinaryOpCtx::Lt(self.consume_token(exp.clone())?),

            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }

    fn parse_bitwise_or(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_bitwise_xor()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        let binary_op_ctx = match next.lexeme_ref() {
            exp if *exp == Lexeme::Stick => BinaryOpCtx::BitOr(self.consume_token(exp.clone())?),

            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }

    fn parse_bitwise_xor(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_bitwise_and()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        let binary_op_ctx = match next.lexeme_ref() {
            exp if *exp == Lexeme::Xor => BinaryOpCtx::BitXor(self.consume_token(exp.clone())?),

            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }

    fn parse_bitwise_and(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_logical_eq()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        let binary_op_ctx = match next.lexeme_ref() {
            exp if *exp == Lexeme::Ampersand => {
                BinaryOpCtx::BitAnd(self.consume_token(exp.clone())?)
            }

            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }

    fn parse_logical_eq(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_logical_less_or_greater()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        let binary_op_ctx = match next.lexeme_ref() {
            exp if *exp == Lexeme::Eq => BinaryOpCtx::Eq(self.consume_token(exp.clone())?),
            exp if *exp == Lexeme::Neq => BinaryOpCtx::Ne(self.consume_token(exp.clone())?),

            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }

    fn parse_logical_less_or_greater(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_shift()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        let binary_op_ctx = match next.lexeme_ref() {
            exp if *exp == Lexeme::Lt => BinaryOpCtx::Lt(self.consume_token(exp.clone())?),
            exp if *exp == Lexeme::Gt => BinaryOpCtx::Gt(self.consume_token(exp.clone())?),
            exp if *exp == Lexeme::Lte => BinaryOpCtx::Le(self.consume_token(exp.clone())?),
            exp if *exp == Lexeme::Gte => BinaryOpCtx::Ge(self.consume_token(exp.clone())?),

            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }

    fn parse_shift(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_add_or_sub()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        let binary_op_ctx = match next.lexeme_ref() {
            exp if *exp == Lexeme::LShift => BinaryOpCtx::Shl(self.consume_token(exp.clone())?),
            exp if *exp == Lexeme::RShift => BinaryOpCtx::Shr(self.consume_token(exp.clone())?),

            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }

    fn parse_add_or_sub(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_mul_or_div()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        let binary_op_ctx = match next.lexeme_ref() {
            exp if *exp == Lexeme::Plus => BinaryOpCtx::Add(self.consume_token(exp.clone())?),
            exp if *exp == Lexeme::Minus => BinaryOpCtx::Sub(self.consume_token(exp.clone())?),

            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }

    fn parse_mul_or_div(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_dot_or_as()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        let binary_op_ctx = match next.lexeme_ref() {
            exp if *exp == Lexeme::Star => BinaryOpCtx::Mul(self.consume_token(exp.clone())?),
            exp if *exp == Lexeme::Slash => BinaryOpCtx::Div(self.consume_token(exp.clone())?),
            exp if *exp == Lexeme::Percent => BinaryOpCtx::Mod(self.consume_token(exp.clone())?),

            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }

    fn parse_dot_or_as(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_scope_resolution()?;

        let Some(next) = self.peek_token() else {
            return Ok(lhs);
        };

        match next.lexeme_ref() {
            lexem if *lexem == Lexeme::KwAs => Ok(ExpressionCtx::Conversion(ConversionCtx {
                expression_ctx: Box::new(lhs),
                as_tkn: self.consume_token(lexem.clone())?,
                type_ctx: Box::new(self.parse_type_ctx()?),
            })),
            lexem if *lexem == Lexeme::Dot => {
                let binary_op_ctx = BinaryOpCtx::Access(self.consume_token(lexem.clone())?);
                Ok(ExpressionCtx::Binary(BinaryCtx {
                    left_ctx: Box::new(lhs),
                    binary_op_ctx,
                    right_ctx: Box::new(self.parse_expression_ctx()?),
                }))
            }

            _ => Err(Message::unexpected_token(
                &next,
                &[Lexeme::Dot, Lexeme::KwAs],
            )),
        }
    }

    fn parse_scope_resolution(&mut self) -> ParseResult<ExpressionCtx> {
        let lhs = self.parse_factor()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let binary_op_ctx = match next.lexeme_ref() {
            lexem if *lexem == Lexeme::Dcolon => {
                BinaryOpCtx::ScopeRes(self.consume_token(lexem.clone())?)
            }

            lexem if *lexem == Lexeme::EndOfLine => {
                self.get_token();
                return Ok(lhs);
            }

            _ => return Err(Message::unexpected_token(&next, &[])),
        };

        Ok(ExpressionCtx::Binary(BinaryCtx {
            left_ctx: Box::new(lhs),
            binary_op_ctx,
            right_ctx: Box::new(self.parse_expression_ctx()?),
        }))
    }
}

/*
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;

    use tanitc_ast::ast::{
        expressions::{BinaryOperation, Expression, ExpressionKind},
        values::{Value, ValueKind},
        Ast,
    };
    use tanitc_lexer::location::Location;
    use tanitc_ty::Type;

    use crate::Parser;

    fn get_location(col: usize) -> Location {
        Location {
            col,
            path: PathBuf::from("text").into(),
            ..Default::default()
        }
    }

    #[test]
    fn binary_expression_test() {
        use tanitc_ident::Ident;

        const SRC_TEXT: &str = "a += 1 * 4 / (1 + a) == 3\n";

        let a_id = Ident::from("a".to_string());

        let expected = Ast::from(Expression {
            location: get_location(4),
            kind: ExpressionKind::Binary {
                operation: BinaryOperation::Assign,
                lhs: Box::new(Ast::from(Value {
                    location: get_location(1),
                    kind: ValueKind::Identifier(a_id),
                })),
                rhs: Box::new(Ast::from(Expression {
                    location: get_location(4),
                    kind: ExpressionKind::Binary {
                        operation: BinaryOperation::Add,
                        lhs: Box::new(Ast::from(Value {
                            location: get_location(1),
                            kind: ValueKind::Identifier(a_id),
                        })),
                        rhs: Box::new(Ast::from(Expression {
                            location: get_location(8),
                            kind: ExpressionKind::Binary {
                                operation: BinaryOperation::Mul,
                                lhs: Box::new(Ast::from(Value {
                                    location: get_location(6),
                                    kind: ValueKind::Integer(1),
                                })),
                                rhs: Box::new(Ast::from(Expression {
                                    location: get_location(23),
                                    kind: ExpressionKind::Binary {
                                        operation: BinaryOperation::LogicalEq,
                                        lhs: Box::new(Ast::from(Expression {
                                            location: get_location(12),
                                            kind: ExpressionKind::Binary {
                                                operation: BinaryOperation::Div,
                                                lhs: Box::new(Ast::from(Value {
                                                    location: get_location(10),
                                                    kind: ValueKind::Integer(4),
                                                })),
                                                rhs: Box::new(Ast::from(Expression {
                                                    location: get_location(17),
                                                    kind: ExpressionKind::Binary {
                                                        operation: BinaryOperation::Add,
                                                        lhs: Box::new(Ast::from(Value {
                                                            location: get_location(15),
                                                            kind: ValueKind::Integer(1),
                                                        })),
                                                        rhs: Box::new(Ast::from(Value {
                                                            location: get_location(19),
                                                            kind: ValueKind::Identifier(a_id),
                                                        })),
                                                    },
                                                })),
                                            },
                                        })),
                                        rhs: Box::new(Ast::from(Value {
                                            location: get_location(25),
                                            kind: ValueKind::Integer(3),
                                        })),
                                    },
                                })),
                            },
                        })),
                    },
                })),
            },
        });

        let mut parser = Parser::from_text(SRC_TEXT);
        let ast = parser.parse_expression().unwrap();

        assert_eq!(ast, expected);
    }

    #[test]
    fn conversion_test() {
        const SRC_TEXT: &str = "45 as f32\n";

        let mut parser = Parser::from_text(SRC_TEXT);

        let expr = parser.parse_expression().unwrap();
        let Ast::Expression(node) = expr else {
            panic!("Expected Ast::Expression, actually: {}", expr.name());
        };

        let ExpressionKind::Conversion { lhs, ty } = &node.kind else {
            panic!("Expected ExpressionKind::Conversion");
        };

        assert!(matches!(
            lhs.as_ref(),
            Ast::Value(Value {
                kind: ValueKind::Integer(45),
                ..
            })
        ));

        assert_eq!(ty.get_type(), Type::F32);
    }
}
*/
