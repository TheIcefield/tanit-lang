use tanitc_ast::ast::{
    expressions::{Expression, ExpressionKind},
    types::{ParsedTypeInfo, TypeSpec},
    variables::{VariableAttributes, VariableDef},
    Ast,
};
use tanitc_attributes::Mutability;
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_ty::Type;

use crate::Parser;

impl Parser {
    pub fn parse_variable_def(&mut self) -> Result<Ast, Message> {
        let next = self.peek_token();
        let location = next.location;

        let is_global = match next.lexem {
            Lexem::KwVar => {
                self.get_token();
                false
            }

            Lexem::KwStatic => {
                self.get_token();
                true
            }

            _ => {
                return Err(Message::unexpected_token(
                    next,
                    &[Lexem::KwVar, Lexem::KwStatic],
                ));
            }
        };

        let next = self.peek_token();
        let mutability = match next.lexem {
            Lexem::KwMut => {
                self.get_token();
                Mutability::Mutable
            }

            Lexem::KwConst => {
                self.get_token();
                Mutability::Immutable
            }

            _ => Mutability::default(),
        };

        let identifier = self.consume_identifier()?;

        let next = self.peek_token();

        let mut var_type: Option<TypeSpec> = None;
        let mut rvalue: Option<Ast> = None;

        if Lexem::Colon == next.lexem {
            self.consume_token(Lexem::Colon)?;

            var_type = Some(self.parse_type_spec()?);
        }

        let next = self.peek_token();

        if Lexem::Assign == next.lexem {
            self.get_token();

            rvalue = Some(self.parse_expression()?);
        }

        if var_type.is_none() && rvalue.is_none() {
            return Err(Message::from_string(
                location,
                format!(
                    "Variable \"{identifier}\" defined without type. Need to specify type or use with rvalue"
                ),
            ));
        }

        if var_type.is_none() && is_global {
            return Err(Message::from_string(
                location,
                format!(
                    "Variable \"{identifier}\" defined without type, but marked as static. Need to specify type"
                ),
            ));
        }

        let var_node = Ast::from(VariableDef {
            location,
            attributes: VariableAttributes::default(),
            identifier,
            var_type: var_type.unwrap_or(TypeSpec {
                location,
                info: ParsedTypeInfo::default(),
                ty: Type::Auto,
            }),
            is_global,
            mutability,
        });

        if let Some(rhs) = rvalue {
            return Ok(Ast::from(Expression {
                location,
                kind: ExpressionKind::new_binary(Lexem::Assign, Box::new(var_node), Box::new(rhs))?,
            }));
        }

        Ok(var_node)
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{
        expressions::{BinaryOperation, Expression, ExpressionKind},
        values::{Value, ValueKind},
        Ast,
    };
    use tanitc_ty::Type;

    use crate::Parser;

    #[test]
    fn notified_vardef_test() {
        const SRC_TEXT: &str = "var a: i32 = 0";

        let mut parser = Parser::from_text(SRC_TEXT).unwrap();

        let res = parser.parse_variable_def().unwrap();
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }

        let Ast::Expression(Expression {
            kind:
                ExpressionKind::Binary {
                    operation,
                    lhs,
                    rhs,
                },
            ..
        }) = &res
        else {
            panic!("Expected Binary expression, actually: {}", res.name());
        };

        let Ast::VariableDef(var_def) = lhs.as_ref() else {
            panic!("Expected VariableDef, actually: {}", lhs.name());
        };

        let Ast::Value(Value {
            kind: ValueKind::Integer(0),
            ..
        }) = rhs.as_ref()
        else {
            panic!("Expected Integer, actually: {}", rhs.name());
        };

        assert_eq!(*operation, BinaryOperation::Assign);

        assert_eq!(var_def.identifier.to_string(), "a");
        assert_eq!(var_def.mutability.is_mutable(), false);
        assert_eq!(var_def.is_global, false);
        assert_eq!(var_def.var_type.get_type(), Type::I32);
    }

    #[test]
    fn notified_mut_vardef_test() {
        const SRC_TEXT: &str = "var mut b: i32 = 0";

        let mut parser = Parser::from_text(SRC_TEXT).unwrap();

        let res = parser.parse_variable_def().unwrap();
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }

        let Ast::Expression(Expression {
            kind:
                ExpressionKind::Binary {
                    operation,
                    lhs,
                    rhs,
                },
            ..
        }) = &res
        else {
            panic!("Expected Binary expression, actually: {}", res.name());
        };

        let Ast::VariableDef(var_def) = lhs.as_ref() else {
            panic!("Expected VariableDef, actually: {}", lhs.name());
        };

        let Ast::Value(Value {
            kind: ValueKind::Integer(0),
            ..
        }) = rhs.as_ref()
        else {
            panic!("Expected Integer, actually: {}", rhs.name());
        };

        assert_eq!(*operation, BinaryOperation::Assign);

        assert_eq!(var_def.identifier.to_string(), "b");
        assert_eq!(var_def.mutability.is_mutable(), true);
        assert_eq!(var_def.is_global, false);
        assert_eq!(var_def.var_type.get_type(), Type::I32);
    }

    #[test]
    fn unnotified_vardef_test() {
        const SRC_TEXT: &str = "var c = 0";

        let mut parser = Parser::from_text(SRC_TEXT).unwrap();

        let res = parser.parse_variable_def().unwrap();
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }

        let Ast::Expression(Expression {
            kind:
                ExpressionKind::Binary {
                    operation,
                    lhs,
                    rhs,
                },
            ..
        }) = &res
        else {
            panic!("Expected Binary expression, actually: {}", res.name());
        };

        let Ast::VariableDef(var_def) = lhs.as_ref() else {
            panic!("Expected VariableDef, actually: {}", lhs.name());
        };

        let Ast::Value(Value {
            kind: ValueKind::Integer(0),
            ..
        }) = rhs.as_ref()
        else {
            panic!("Expected Integer, actually: {}", rhs.name());
        };

        assert_eq!(*operation, BinaryOperation::Assign);

        assert_eq!(var_def.identifier.to_string(), "c");
        assert_eq!(var_def.mutability.is_mutable(), false);
        assert_eq!(var_def.is_global, false);
        assert_eq!(var_def.var_type.get_type(), Type::Auto);
    }

    #[test]
    fn unnotified_mut_vardef_test() {
        const SRC_TEXT: &str = "var mut d = 0";

        let mut parser = Parser::from_text(SRC_TEXT).unwrap();

        let res = parser.parse_variable_def().unwrap();
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }

        let Ast::Expression(Expression {
            kind:
                ExpressionKind::Binary {
                    operation,
                    lhs,
                    rhs,
                },
            ..
        }) = &res
        else {
            panic!("Expected Binary expression, actually: {}", res.name());
        };

        let Ast::VariableDef(var_def) = lhs.as_ref() else {
            panic!("Expected VariableDef, actually: {}", lhs.name());
        };

        let Ast::Value(Value {
            kind: ValueKind::Integer(0),
            ..
        }) = rhs.as_ref()
        else {
            panic!("Expected Integer, actually: {}", rhs.name());
        };

        assert_eq!(*operation, BinaryOperation::Assign);

        assert_eq!(var_def.identifier.to_string(), "d");
        assert_eq!(var_def.mutability.is_mutable(), true);
        assert_eq!(var_def.is_global, false);
        assert_eq!(var_def.var_type.get_type(), Type::Auto);
    }

    #[test]
    fn notified_vardec_test() {
        const SRC_TEXT: &str = "var e: i32";

        let mut parser = Parser::from_text(SRC_TEXT).unwrap();

        let res = parser.parse_variable_def().unwrap();
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }

        let Ast::VariableDef(var_def) = &res else {
            panic!("Expected VariableDef, actually: {}", res.name());
        };

        assert_eq!(var_def.identifier.to_string(), "e");
        assert_eq!(var_def.mutability.is_mutable(), false);
        assert_eq!(var_def.is_global, false);
        assert_eq!(var_def.var_type.get_type(), Type::I32);
    }

    #[test]
    fn notified_mut_vardec_test() {
        const SRC_TEXT: &str = "var mut f: f32";

        let mut parser = Parser::from_text(SRC_TEXT).unwrap();

        let res = parser.parse_variable_def().unwrap();
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }

        let Ast::VariableDef(var_def) = &res else {
            panic!("Expected VariableDef, actually: {}", res.name());
        };

        assert_eq!(var_def.identifier.to_string(), "f");
        assert_eq!(var_def.mutability.is_mutable(), true);
        assert_eq!(var_def.is_global, false);
        assert_eq!(var_def.var_type.get_type(), Type::F32);
    }

    #[test]
    fn unnotified_vardec_bad_test() {
        const SRC_TEXT: &str = "var e";
        const EXPECTED_ERR: &str =
            "Variable \"e\" defined without type. Need to specify type or use with rvalue";

        let mut parser = Parser::from_text(SRC_TEXT).unwrap();

        let msg = parser.parse_variable_def().err().unwrap();
        assert_eq!(msg.text, EXPECTED_ERR);
    }

    #[test]
    fn notified_static_def_test() {
        const SRC_TEXT: &str = "static A: i32 = 0";

        let mut parser = Parser::from_text(SRC_TEXT).unwrap();

        let res = parser.parse_variable_def().unwrap();
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }

        let Ast::Expression(Expression {
            kind:
                ExpressionKind::Binary {
                    operation,
                    lhs,
                    rhs,
                },
            ..
        }) = &res
        else {
            panic!("Expected Binary expression, actually: {}", res.name());
        };

        let Ast::VariableDef(var_def) = lhs.as_ref() else {
            panic!("Expected VariableDef, actually: {}", lhs.name());
        };

        let Ast::Value(Value {
            kind: ValueKind::Integer(0),
            ..
        }) = rhs.as_ref()
        else {
            panic!("Expected Integer, actually: {}", rhs.name());
        };

        assert_eq!(*operation, BinaryOperation::Assign);

        assert_eq!(var_def.identifier.to_string(), "A");
        assert_eq!(var_def.mutability.is_mutable(), false);
        assert_eq!(var_def.is_global, true);
        assert_eq!(var_def.var_type.get_type(), Type::I32);
    }

    #[test]
    fn notified_static_mut_def_test() {
        const SRC_TEXT: &str = "static mut B: i32 = 0";

        let mut parser = Parser::from_text(SRC_TEXT).unwrap();

        let res = parser.parse_variable_def().unwrap();
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }

        let Ast::Expression(Expression {
            kind:
                ExpressionKind::Binary {
                    operation,
                    lhs,
                    rhs,
                },
            ..
        }) = &res
        else {
            panic!("Expected Binary expression, actually: {}", res.name());
        };

        let Ast::VariableDef(var_def) = lhs.as_ref() else {
            panic!("Expected VariableDef, actually: {}", lhs.name());
        };

        let Ast::Value(Value {
            kind: ValueKind::Integer(0),
            ..
        }) = rhs.as_ref()
        else {
            panic!("Expected Integer, actually: {}", rhs.name());
        };

        assert_eq!(*operation, BinaryOperation::Assign);

        assert_eq!(var_def.identifier.to_string(), "B");
        assert_eq!(var_def.mutability.is_mutable(), true);
        assert_eq!(var_def.is_global, true);
        assert_eq!(var_def.var_type.get_type(), Type::I32);
    }

    #[test]
    fn unnotified_static_def_bad_test() {
        const SRC_TEXT: &str = "static C = 0";
        const EXPECTED_ERR: &str =
            "Variable \"C\" defined without type, but marked as static. Need to specify type";

        let mut parser = Parser::from_text(SRC_TEXT).unwrap();

        let msg = parser.parse_variable_def().err().unwrap();
        assert_eq!(msg.text, EXPECTED_ERR);
    }
}
