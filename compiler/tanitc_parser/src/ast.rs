use tanitc_ast::{
    Ast, CallParam, Expression, ExpressionKind, FunctionDef, StructDef, TypeInfo, TypeSpec, Value,
    ValueKind, VariableDef, VariantDef, VariantField,
};
use tanitc_ident::Ident;
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_ty::Type;

use crate::Parser;

// Alias
impl Parser {
    pub fn parse_alias_def(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::AliasDef;
        let mut node = AliasDef {
            location: self.consume_token(Lexem::KwAlias)?.location,
            identifier: self.consume_identifier()?,
            ..Default::default()
        };

        self.consume_token(Lexem::Assign)?;

        node.value = self.parse_type_spec()?;

        Ok(Ast::from(node))
    }
}

// Branches
impl Parser {
    fn parse_branch(&mut self) -> Result<Ast, Message> {
        let next = self.peek_token();
        match next.lexem {
            Lexem::KwLoop => self.parse_loop(),
            Lexem::KwWhile => self.parse_while(),
            Lexem::KwIf => self.parse_if(),
            Lexem::KwElse => self.parse_else(),
            _ => Err(Message::unexpected_token(
                next,
                &[Lexem::KwLoop, Lexem::KwWhile, Lexem::KwIf, Lexem::KwElse],
            )),
        }
    }

    fn parse_loop(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::{Branch, BranchKind};

        let location = self.consume_token(Lexem::KwLoop)?.location;

        let body = Box::new(self.parse_local_block()?);

        Ok(Ast::from(Branch {
            location,
            kind: BranchKind::Loop { body },
        }))
    }

    fn parse_while(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::{Branch, BranchKind};

        let location = self.consume_token(Lexem::KwWhile)?.location;

        let condition = Box::new(self.parse_expression()?);
        let body = Box::new(self.parse_local_block()?);

        Ok(Ast::from(Branch {
            location,
            kind: BranchKind::While { body, condition },
        }))
    }

    fn parse_if(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::{Branch, BranchKind};

        let location = self.consume_token(Lexem::KwIf)?.location;

        let condition = Box::new(self.parse_expression()?);
        let body = Box::new(self.parse_local_block()?);

        Ok(Ast::from(Branch {
            location,
            kind: BranchKind::If { condition, body },
        }))
    }

    fn parse_else(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::{Branch, BranchKind};

        let location = self.consume_token(Lexem::KwElse)?.location;

        let body = Box::new(if Lexem::KwIf == self.peek_token().lexem {
            self.parse_if()?
        } else {
            self.parse_local_block()?
        });

        Ok(Ast::from(Branch {
            location,
            kind: BranchKind::Else { body },
        }))
    }
}

// Control flows
impl Parser {
    pub fn parse_control_flow(&mut self) -> Result<Ast, Message> {
        let next = self.peek_token();
        match next.lexem {
            Lexem::KwBreak => self.parse_break(),
            Lexem::KwContinue => self.parse_continue(),
            Lexem::KwReturn => self.parse_return(),
            _ => Err(Message::unexpected_token(
                next,
                &[Lexem::KwBreak, Lexem::KwContinue, Lexem::KwReturn],
            )),
        }
    }

    pub fn parse_break(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::{ControlFlow, ControlFlowKind};

        let location = self.consume_token(Lexem::KwBreak)?.location;

        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(false);

        let mut node = ControlFlow {
            location,
            kind: ControlFlowKind::Break { ret: None },
        };

        match self.peek_token().lexem {
            Lexem::EndOfLine => {}
            _ => {
                let expr = self.parse_expression()?;

                node.kind = ControlFlowKind::Break {
                    ret: Some(Box::new(expr)),
                }
            }
        }

        self.set_ignore_nl_option(old_opt);

        Ok(Ast::from(node))
    }

    pub fn parse_continue(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::{ControlFlow, ControlFlowKind};

        let location = self.consume_token(Lexem::KwContinue)?.location;

        Ok(Ast::from(ControlFlow {
            location,
            kind: ControlFlowKind::Continue,
        }))
    }

    pub fn parse_return(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::{ControlFlow, ControlFlowKind};

        let location = self.consume_token(Lexem::KwReturn)?.location;

        let mut node = ControlFlow {
            location,
            kind: ControlFlowKind::Return { ret: None },
        };

        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(false);

        match self.peek_token().lexem {
            Lexem::EndOfLine => {}
            _ => {
                let expr = self.parse_expression()?;

                node.kind = ControlFlowKind::Return {
                    ret: Some(Box::new(expr)),
                }
            }
        }

        self.set_ignore_nl_option(old_opt);

        Ok(Ast::from(node))
    }
}

// Enum definition
impl Parser {
    pub fn parse_enum_def(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::EnumDef;

        let mut node = EnumDef::default();

        self.parse_enum_header(&mut node)?;
        self.parse_enum_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_enum_header(&mut self, enum_def: &mut tanitc_ast::EnumDef) -> Result<(), Message> {
        enum_def.location = self.consume_token(Lexem::KwEnum)?.location;
        enum_def.identifier = self.consume_identifier()?;

        Ok(())
    }

    fn parse_enum_body(&mut self, enum_def: &mut tanitc_ast::EnumDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;
        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(false);
        self.parse_enum_body_internal(enum_def)?;
        self.set_ignore_nl_option(old_opt);

        self.consume_token(Lexem::Rcb)?;

        Ok(())
    }

    fn parse_enum_body_internal(
        &mut self,
        enum_def: &mut tanitc_ast::EnumDef,
    ) -> Result<(), Message> {
        loop {
            let next = self.peek_token();

            match &next.lexem {
                Lexem::Rcb => break,
                Lexem::EndOfLine => {
                    self.get_token();
                    continue;
                }
                Lexem::Identifier(id) => {
                    let identifier = self.consume_identifier()?;

                    let value = if Lexem::Colon == self.peek_token().lexem {
                        self.consume_token(Lexem::Colon)?;

                        let token = self.consume_integer()?;
                        let value = if let Lexem::Integer(value) = token.lexem {
                            match value.parse::<usize>() {
                                Ok(value) => value,
                                Err(err) => {
                                    return Err(Message::parse_int_error(token.location, err))
                                }
                            }
                        } else {
                            unreachable!()
                        };

                        Some(value)
                    } else {
                        None
                    };

                    if enum_def.fields.contains_key(&identifier) {
                        self.error(Message::new(
                            next.location,
                            &format!("Enum has already field with identifier \"{}\"", id),
                        ));
                        continue;
                    }

                    enum_def.fields.insert(identifier, value);

                    self.consume_new_line()?;
                }

                Lexem::Lcb => {
                    return Err(Message::new(
                        next.location,
                        &format!(
                            "{}\nHelp: {}{}",
                            "Unexpected token: \"{\" during parsing enum fields.",
                            "If you tried to declare struct-like field, place \"{\" ",
                            "in the same line with name of the field."
                        ),
                    ));
                }

                _ => {
                    return Err(Message::unexpected_token(next, &[]));
                }
            }
        }

        Ok(())
    }
}

// Expression
impl Parser {
    pub fn parse_expression(&mut self) -> Result<Ast, Message> {
        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(false);
        let expr = self.parse_assign()?;
        self.set_ignore_nl_option(old_opt);

        if let Ast::Expression(node) = &expr {
            let location = node.location;

            if let ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } = &node.kind
            {
                let new_op = match operation {
                    Lexem::AddAssign => Some(Lexem::Plus),
                    Lexem::SubAssign => Some(Lexem::Minus),
                    Lexem::MulAssign => Some(Lexem::Star),
                    Lexem::DivAssign => Some(Lexem::Slash),
                    Lexem::ModAssign => Some(Lexem::Percent),
                    Lexem::XorAssign => Some(Lexem::Xor),
                    Lexem::AndAssign => Some(Lexem::Ampersand),
                    Lexem::OrAssign => Some(Lexem::Stick),
                    Lexem::LShiftAssign => Some(Lexem::LShift),
                    Lexem::RShiftAssign => Some(Lexem::RShift),
                    _ => None,
                };

                if let Some(new_op) = new_op {
                    return Ok(Ast::from(Expression {
                        location,
                        kind: ExpressionKind::Binary {
                            operation: Lexem::Assign,
                            lhs: lhs.clone(),
                            rhs: Box::new(Ast::from(Expression {
                                location,
                                kind: ExpressionKind::Binary {
                                    operation: new_op,
                                    lhs: lhs.clone(),
                                    rhs: rhs.clone(),
                                },
                            })),
                        },
                    }));
                }
            }
        }

        Ok(expr)
    }

    pub fn parse_factor(&mut self) -> Result<Ast, Message> {
        let next = self.peek_token();
        let location = next.location;

        match &next.lexem {
            Lexem::Plus | Lexem::Minus | Lexem::Ampersand | Lexem::Star | Lexem::Not => {
                self.get_token();
                let operation = next.lexem;
                let node = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Unary { operation, node },
                }))
            }
            Lexem::Integer(_) => Ok(Ast::from(Value {
                location,
                kind: ValueKind::Integer(
                    self.consume_integer()?.lexem.get_int().unwrap_or_default(),
                ),
            })),

            Lexem::Decimal(_) => Ok(Ast::from(Value {
                location,
                kind: ValueKind::Decimal(
                    self.consume_decimal()?.lexem.get_dec().unwrap_or_default(),
                ),
            })),

            Lexem::Identifier(_) => {
                let identifier = self.consume_identifier()?;

                let next = self.peek_token();
                if next.lexem == Lexem::LParen {
                    // if call
                    let arguments = self.parse_call_params()?;

                    return Ok(Ast::from(Value {
                        location,
                        kind: ValueKind::Call {
                            identifier,
                            arguments,
                        },
                    }));
                } else if next.lexem == Lexem::Dcolon {
                    // if ::
                    self.get_token();

                    let operation = next.lexem;

                    let lhs = Box::new(Ast::from(Value {
                        location,
                        kind: ValueKind::Identifier(identifier),
                    }));

                    let rhs = Box::new(self.parse_factor()?);

                    return Ok(Ast::from(Expression {
                        location,
                        kind: ExpressionKind::Binary {
                            operation,
                            lhs,
                            rhs,
                        },
                    }));
                } else if next.lexem == Lexem::Lcb {
                    // if struct
                    let components = self.parse_struct_value()?;

                    return Ok(Ast::from(Value {
                        location,
                        kind: ValueKind::Struct {
                            identifier,
                            components,
                        },
                    }));
                }

                Ok(Ast::from(Value {
                    location,
                    kind: ValueKind::Identifier(identifier),
                }))
            }

            Lexem::LParen => {
                self.consume_token(Lexem::LParen)?;

                /* If parsed `()` then we return empty tuple */
                if self.peek_token().lexem == Lexem::RParen {
                    self.consume_token(Lexem::RParen)?;
                    return Ok(Ast::from(Value {
                        location,
                        kind: ValueKind::Tuple {
                            components: Vec::new(),
                        },
                    }));
                }

                let mut components = Vec::<Ast>::new();

                let expr = self.parse_expression()?;

                let is_tuple = match &expr {
                    Ast::Expression { .. } => false,
                    Ast::Value { .. } => true,
                    _ => return Err(Message::new(next.location, "Unexpected node parsed")),
                };

                /* If parsed one expression, we return expression */
                if !is_tuple {
                    self.consume_token(Lexem::RParen)?;
                    return Ok(expr);
                }

                /* else try parse tuple */
                components.push(expr);

                loop {
                    let next = self.peek_token();

                    if next.lexem == Lexem::RParen {
                        self.consume_token(Lexem::RParen)?;
                        break;
                    } else if next.lexem == Lexem::Comma {
                        self.consume_token(Lexem::Comma)?;
                        components.push(self.parse_expression()?);
                    } else {
                        return Err(Message::new(
                            next.location,
                            &format!("Unexpected token \"{}\" within tuple", next),
                        ));
                    }
                }

                Ok(Ast::from(Value {
                    location,
                    kind: ValueKind::Tuple { components },
                }))
            }

            Lexem::Lsb => self.parse_array_value(),

            _ => Err(Message::new(
                next.location,
                &format!("Unexpected token \"{}\" within expression", next),
            )),
        }
    }
}

// Expression internal
impl Parser {
    fn parse_assign(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_logical_or()?;

        let next = self.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Assign
            | Lexem::AddAssign
            | Lexem::SubAssign
            | Lexem::MulAssign
            | Lexem::DivAssign
            | Lexem::ModAssign
            | Lexem::OrAssign
            | Lexem::AndAssign
            | Lexem::XorAssign
            | Lexem::LShiftAssign
            | Lexem::RShiftAssign => {
                self.get_token();
                let operation = next.lexem;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_or(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_logical_and()?;

        let next = self.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Or => {
                self.get_token();
                let operation = Lexem::Or;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_and(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_bitwise_or()?;

        let next = self.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::And => {
                self.get_token();
                let operation = Lexem::And;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_or(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_bitwise_xor()?;

        let next = self.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Stick => {
                self.get_token();
                let operation = Lexem::Stick;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_xor(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_bitwise_and()?;

        let next = self.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Xor => {
                self.get_token();
                let operation = Lexem::Xor;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_and(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_logical_eq()?;

        let next = self.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Ampersand => {
                self.get_token();
                let operation = Lexem::Ampersand;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_eq(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_logical_less_or_greater()?;

        let next = self.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Eq | Lexem::Neq => {
                self.get_token();
                let operation = next.lexem;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_less_or_greater(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_shift()?;

        let next = self.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Lt | Lexem::Lte | Lexem::Gt | Lexem::Gte => {
                self.get_token();
                let operation = next.lexem;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_shift(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_add_or_sub()?;

        let next = self.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::LShift | Lexem::RShift => {
                self.get_token();
                let operation = next.lexem;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_add_or_sub(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_mul_or_div()?;

        let next = self.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Plus | Lexem::Minus => {
                self.get_token();
                let operation = next.lexem;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_mul_or_div(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_dot_or_as()?;

        let next = self.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Star | Lexem::Slash | Lexem::Percent => {
                self.get_token();
                let operation = next.lexem;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_dot_or_as(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_factor()?;

        let next = self.peek_token();

        match next.lexem {
            Lexem::Dot | Lexem::KwAs => {
                self.get_token();
                let operation = next.lexem;
                let is_conversion = Lexem::KwAs == operation;

                if is_conversion {
                    let location = self.peek_token().location;
                    return Ok(Ast::from(Expression {
                        location,
                        kind: ExpressionKind::Conversion {
                            lhs: Box::new(lhs),
                            ty: self.parse_type_spec()?,
                        },
                    }));
                }

                Ok(Ast::from(Expression {
                    location: next.location,
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs: Box::new(self.parse_expression()?),
                    },
                }))
            }
            _ => Ok(lhs),
        }
    }
}

// Function definition
impl Parser {
    pub fn parse_func_def(&mut self) -> Result<Ast, Message> {
        let mut node = FunctionDef::default();

        self.parse_func_header(&mut node)?;
        self.parse_func_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_func_header(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        func_def.location = self.consume_token(Lexem::KwFunc)?.location;
        func_def.identifier = self.consume_identifier()?;

        self.parse_func_header_params(func_def)?;

        let next = self.peek_token();
        func_def.return_type = if Lexem::Arrow == next.lexem {
            self.get_token();
            self.parse_type_spec()?
        } else {
            TypeSpec {
                location: next.location,
                info: TypeInfo::default(),
                ty: Type::unit(),
            }
        };

        Ok(())
    }

    fn parse_func_header_params(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        self.consume_token(Lexem::LParen)?;

        loop {
            let next = self.peek_token();

            if next.is_identifier() {
                func_def
                    .parameters
                    .push(Ast::VariableDef(self.parse_func_param()?));

                let next = self.peek_token();
                if next.lexem == Lexem::Comma {
                    self.get_token();
                } else if next.lexem == Lexem::RParen {
                    continue;
                } else {
                    return Err(Message::unexpected_token(next, &[]));
                }
            } else if next.lexem == Lexem::RParen {
                self.get_token();
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        Ok(())
    }

    fn parse_func_body(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        let next = self.peek_token();
        match next.lexem {
            Lexem::Lcb => {
                func_def.body = Some(Box::new(self.parse_local_block()?));
            }

            _ => {
                return Err(Message::unexpected_token(
                    next,
                    &[Lexem::Lcb, Lexem::EndOfLine],
                ));
            }
        }

        Ok(())
    }

    fn parse_func_param(&mut self) -> Result<VariableDef, Message> {
        let location = self.peek_token().location;
        let identifier = self.consume_identifier()?;

        self.consume_token(Lexem::Colon)?;

        let var_type = self.parse_type_spec()?;

        Ok(VariableDef {
            location,
            identifier,
            var_type,
            is_global: false,
            is_mutable: true,
        })
    }
}

// Module definition
impl Parser {
    pub fn parse_module_def(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::ModuleDef;

        let mut node = ModuleDef::default();

        self.parse_module_header(&mut node)?;
        self.parse_module_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_module_header(&mut self, mod_def: &mut tanitc_ast::ModuleDef) -> Result<(), Message> {
        let next = self.peek_token();
        mod_def.location = next.location;

        if Lexem::KwDef == next.lexem {
            self.consume_token(Lexem::KwDef)?;
            mod_def.is_external = true;
        }

        self.consume_token(Lexem::KwModule)?;

        mod_def.identifier = self.consume_identifier()?;

        Ok(())
    }

    fn parse_module_body(&mut self, mod_def: &mut tanitc_ast::ModuleDef) -> Result<(), Message> {
        if !mod_def.is_external {
            self.parse_module_body_internal(mod_def)
        } else {
            Ok(())
        }
    }

    fn parse_module_body_internal(
        &mut self,
        mod_def: &mut tanitc_ast::ModuleDef,
    ) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;

        let block = self.parse_global_block()?;

        self.consume_token(Lexem::Rcb)?;

        if let Ast::Block(node) = block {
            mod_def.body = Some(node);
        } else {
            return Err(Message::unreachable(mod_def.location));
        }

        Ok(())
    }
}

// Block
impl Parser {
    pub fn parse_global_block(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::Block;

        let mut node = Block::default();

        self.parse_block_internal(&mut node)?;
        node.is_global = true;

        Ok(Ast::from(node))
    }

    pub fn parse_local_block(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::Block;

        let mut node = Block::default();

        self.consume_token(Lexem::Lcb)?;

        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(false);

        self.parse_block_internal(&mut node)?;
        node.is_global = false;

        self.consume_token(Lexem::Rcb)?;

        self.set_ignore_nl_option(old_opt);

        Ok(Ast::from(node))
    }

    fn parse_block_internal(&mut self, block: &mut tanitc_ast::Block) -> Result<(), Message> {
        loop {
            let next = self.peek_token();

            let statement = match next.lexem {
                Lexem::Rcb | Lexem::EndOfFile => {
                    break;
                }

                Lexem::EndOfLine => {
                    self.get_token();
                    continue;
                }

                Lexem::KwDef | Lexem::KwModule => self.parse_module_def(),

                Lexem::KwFunc => self.parse_func_def(),

                Lexem::KwEnum => self.parse_enum_def(),

                Lexem::KwStruct => self.parse_struct_def(),

                Lexem::KwVariant => self.parse_variant_def(),

                Lexem::KwVar | Lexem::KwStatic => self.parse_variable_def(),

                Lexem::KwAlias => self.parse_alias_def(),

                Lexem::Identifier(_) | Lexem::Integer(_) | Lexem::Decimal(_) => {
                    self.parse_expression()
                }

                Lexem::KwLoop | Lexem::KwWhile | Lexem::KwIf | Lexem::KwElse => self.parse_branch(),

                Lexem::KwReturn | Lexem::KwBreak | Lexem::KwContinue => self.parse_control_flow(),

                Lexem::Lcb => self.parse_local_block(),

                _ => {
                    self.skip_until(&[Lexem::EndOfLine]);
                    self.get_token();

                    self.error(Message::unexpected_token(next, &[]));
                    continue;
                }
            };

            match statement {
                Ok(child) => block.statements.push(child),
                Err(err) => self.error(err),
            }
        }

        Ok(())
    }
}

// Struct definition
impl Parser {
    pub fn parse_struct_def(&mut self) -> Result<Ast, Message> {
        let mut node = StructDef::default();

        self.parse_struct_header(&mut node)?;
        self.parse_struct_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_struct_header(
        &mut self,
        struct_def: &mut tanitc_ast::StructDef,
    ) -> Result<(), Message> {
        struct_def.location = self.consume_token(Lexem::KwStruct)?.location;
        struct_def.identifier = self.consume_identifier()?;
        Ok(())
    }

    fn parse_struct_body(&mut self, struct_def: &mut tanitc_ast::StructDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;

        self.parse_struct_body_internal(struct_def)?;

        self.consume_token(Lexem::Rcb)?;

        Ok(())
    }

    fn parse_struct_body_internal(
        &mut self,
        struct_def: &mut tanitc_ast::StructDef,
    ) -> Result<(), Message> {
        loop {
            let next = self.peek_token();

            match &next.lexem {
                Lexem::Rcb => break,

                Lexem::EndOfLine => {
                    self.get_token();
                    continue;
                }

                Lexem::KwStruct => {
                    struct_def.internals.push(self.parse_struct_def()?);
                }

                Lexem::KwVariant => {
                    struct_def.internals.push(self.parse_variant_def()?);
                }

                Lexem::Identifier(id) => {
                    let identifier = self.consume_identifier()?;

                    if struct_def.fields.contains_key(&identifier) {
                        self.error(Message::new(
                            next.location,
                            &format!("Struct has already field with identifier {}", id),
                        ));
                        continue;
                    }

                    self.consume_token(Lexem::Colon)?;

                    struct_def
                        .fields
                        .insert(identifier, self.parse_type_spec()?);
                }

                _ => {
                    return Err(Message::new(
                        next.location,
                        "Unexpected token when parsing struct fields",
                    ));
                }
            }
        }

        Ok(())
    }
}

// Type specification
impl Parser {
    fn parse_type_spec(&mut self) -> Result<TypeSpec, Message> {
        let location = self.peek_token().location;
        let (ty, info) = self.parse_type()?;

        Ok(TypeSpec { location, info, ty })
    }

    fn parse_type(&mut self) -> Result<(Type, TypeInfo), Message> {
        let mut info = TypeInfo::default();
        let next = self.peek_token();

        if self.peek_token().lexem == Lexem::Ampersand {
            info.is_mut = false;
            self.get_token();

            if matches!(self.peek_token().lexem, Lexem::KwMut) {
                info.is_mut = true;
                self.get_token();
            }

            let (ref_to, _) = self.parse_type()?;

            return Ok((Type::Ref(Box::new(ref_to)), info));
        }

        if next.lexem == Lexem::Star {
            info.is_mut = false;
            self.get_token();

            if matches!(self.peek_token().lexem, Lexem::KwMut) {
                info.is_mut = true;
                self.get_token();
            }

            let (ptr_to, _) = self.parse_type()?;

            return Ok((Type::Ptr(Box::new(ptr_to)), info));
        }

        if next.lexem == Lexem::LParen {
            return self.parse_tuple_def();
        }

        if next.lexem == Lexem::Lsb {
            return self.parse_array_def();
        }

        let identifier = self.consume_identifier()?;
        let id_str: String = identifier.into();

        match &id_str[..] {
            "!" => return Ok((Type::Never, info)),
            "bool" => return Ok((Type::Bool, info)),
            "i8" => return Ok((Type::I8, info)),
            "i16" => return Ok((Type::I16, info)),
            "i32" => return Ok((Type::I32, info)),
            "i64" => return Ok((Type::I64, info)),
            "i128" => return Ok((Type::I128, info)),
            "u8" => return Ok((Type::U8, info)),
            "u16" => return Ok((Type::U16, info)),
            "u32" => return Ok((Type::U32, info)),
            "u64" => return Ok((Type::U64, info)),
            "u128" => return Ok((Type::U128, info)),
            "f32" => return Ok((Type::F32, info)),
            "f64" => return Ok((Type::F64, info)),
            "str" => return Ok((Type::Str, info)),
            _ => {}
        }

        if self.peek_singular().lexem == Lexem::Lt {
            return Ok((
                Type::Template {
                    identifier,
                    generics: self.parse_template_generics()?,
                },
                info,
            ));
        }

        Ok((Type::Custom(id_str), info))
    }

    fn parse_tuple_def(&mut self) -> Result<(Type, TypeInfo), Message> {
        self.consume_token(Lexem::LParen)?;

        let mut children = Vec::<Type>::new();
        loop {
            if self.peek_token().lexem == Lexem::RParen {
                break;
            }

            let (child, _) = self.parse_type()?;
            children.push(child);

            if self.peek_token().lexem == Lexem::Comma {
                self.get_token();
                continue;
            }
        }

        self.consume_token(Lexem::RParen)?;

        Ok((Type::Tuple(children), TypeInfo::default()))
    }

    fn parse_array_def(&mut self) -> Result<(Type, TypeInfo), Message> {
        self.consume_token(Lexem::Lsb)?;

        let (value_type, _) = self.parse_type()?;

        if self.peek_token().lexem == Lexem::Colon {
            self.get_token();

            // size = Some(Box::new(self.parse_expression()?));
        }

        self.consume_token(Lexem::Rsb)?;

        Ok((
            Type::Array {
                size: None,
                value_type: Box::new(value_type),
            },
            TypeInfo::default(),
        ))
    }

    fn parse_template_generics(&mut self) -> Result<Vec<Type>, Message> {
        self.consume_token(Lexem::Lt)?;

        let mut children = Vec::<Type>::new();
        loop {
            let (child, _) = self.parse_type()?;
            children.push(child);

            let next = self.peek_singular();
            if next.lexem == Lexem::Gt {
                break;
            } else {
                self.consume_token(Lexem::Comma)?;
            }
        }

        self.get_singular();

        Ok(children)
    }
}

// Values
impl Parser {
    fn parse_call_params(&mut self) -> Result<Vec<CallParam>, Message> {
        let _ = self.consume_token(Lexem::LParen)?.location;

        let mut args = Vec::<CallParam>::new();

        let mut i = 0;
        loop {
            let next = self.peek_token();

            if next.lexem == Lexem::RParen {
                break;
            }

            let expr = self.parse_expression()?;

            let param_id = if let Ast::Value(Value {
                location: _,
                kind: ValueKind::Identifier(id),
            }) = &expr
            {
                if self.peek_token().lexem == Lexem::Colon {
                    self.consume_token(Lexem::Colon)?;
                    Some(id)
                } else {
                    None
                }
            } else {
                None
            };

            let param = if let Some(id) = param_id {
                CallParam::Notified(*id, Box::new(self.parse_expression()?))
            } else {
                CallParam::Positional(i, Box::new(expr))
            };

            args.push(param);

            i += 1;

            let next = self.peek_token();
            if next.lexem == Lexem::Comma {
                // continue parsing if ','
                self.get_token();
                continue;
            } else if next.lexem == Lexem::RParen {
                // end parsing if ')'
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        self.consume_token(Lexem::RParen)?;

        Ok(args)
    }

    fn parse_array_value(&mut self) -> Result<Ast, Message> {
        let location = self.consume_token(Lexem::Lsb)?.location;

        let mut components = Vec::<Ast>::new();

        loop {
            let next = self.peek_token();

            if next.lexem == Lexem::Rsb {
                break;
            }
            components.push(self.parse_expression()?);

            let next = self.peek_token();
            if next.lexem == Lexem::Comma {
                // continue parsing if ','
                self.get_token();
                continue;
            } else if next.lexem == Lexem::Rsb {
                // end parsing if ']'
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        self.consume_token(Lexem::Rsb)?;

        Ok(Ast::from(Value {
            location,
            kind: ValueKind::Array { components },
        }))
    }

    fn parse_struct_value(&mut self) -> Result<Vec<(Ident, Ast)>, Message> {
        self.consume_token(Lexem::Lcb)?;

        let mut components = Vec::<(Ident, Ast)>::new();

        loop {
            let next = self.peek_token();

            if next.lexem == Lexem::Rcb {
                break;
            }

            let identifier = self.consume_identifier()?;

            self.consume_token(Lexem::Colon)?;

            components.push((identifier, self.parse_expression()?));

            let next = self.peek_token();
            if next.lexem == Lexem::Comma {
                // continue parsing if ','
                self.get_token();
                continue;
            } else if next.lexem == Lexem::Rcb {
                // end parsing if '}'
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        self.consume_token(Lexem::Rcb)?;

        Ok(components)
    }
}

// Variable definition
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
        let is_mutable = match next.lexem {
            Lexem::KwMut => {
                self.get_token();
                true
            }

            Lexem::KwConst => {
                self.get_token();
                false
            }

            _ => false,
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
            return Err(Message::new(
                location,
                &format!(
                    "Variable {} defined without type. Need to specify type or use with rvalue",
                    identifier
                ),
            ));
        }

        if var_type.is_none() && is_global {
            return Err(Message::new(
                location,
                &format!(
                    "Variable {} defined without type, but marked as static. Need to specify type",
                    identifier
                ),
            ));
        }

        let var_node = Ast::from(VariableDef {
            location,
            identifier,
            var_type: var_type.unwrap_or(TypeSpec {
                location,
                info: TypeInfo::default(),
                ty: Type::Auto,
            }),
            is_global,
            is_mutable,
        });

        if let Some(rhs) = rvalue {
            return Ok(Ast::from(Expression {
                location,
                kind: ExpressionKind::Binary {
                    operation: Lexem::Assign,
                    lhs: Box::new(var_node),
                    rhs: Box::new(rhs),
                },
            }));
        }

        Ok(var_node)
    }
}

// Variant definition
impl Parser {
    pub fn parse_variant_def(&mut self) -> Result<Ast, Message> {
        let mut node = VariantDef::default();

        self.parse_variant_header(&mut node)?;
        self.parse_variant_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_variant_header(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        variant_def.location = self.consume_token(Lexem::KwVariant)?.location;
        variant_def.identifier = self.consume_identifier()?;

        Ok(())
    }

    fn parse_variant_body(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;
        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(false);
        self.parse_variant_body_internal(variant_def)?;
        self.set_ignore_nl_option(old_opt);

        self.consume_token(Lexem::Rcb)?;

        Ok(())
    }

    fn parse_variant_body_internal(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        loop {
            let next = self.peek_token();

            match &next.lexem {
                Lexem::Rcb => break,

                Lexem::EndOfLine => {
                    self.get_token();
                    continue;
                }

                Lexem::KwStruct => variant_def.internals.push(self.parse_struct_def()?),

                Lexem::KwVariant => variant_def.internals.push(self.parse_variant_def()?),

                Lexem::Identifier(id) => {
                    let identifier = self.consume_identifier()?;

                    if variant_def.fields.contains_key(&identifier) {
                        self.error(Message::new(
                            next.location,
                            &format!("Enum has already field with identifier \"{}\"", id),
                        ));
                        continue;
                    }

                    variant_def
                        .fields
                        .insert(identifier, self.parse_variant_field()?);

                    self.consume_new_line()?;
                }

                Lexem::Lcb => {
                    return Err(Message::new(
                        next.location,
                        &format!(
                            "{}\nHelp: {}{}",
                            "Unexpected token: \"{\" during parsing enum fields.",
                            "If you tried to declare struct-like field, place \"{\" ",
                            "in the same line with name of the field."
                        ),
                    ));
                }

                _ => {
                    return Err(Message::unexpected_token(next, &[]));
                }
            }
        }

        Ok(())
    }
}

// Variant field
impl Parser {
    fn parse_variant_field(&mut self) -> Result<VariantField, Message> {
        let mut node = VariantField::default();

        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(false);

        self.parse_variant_field_internal(&mut node)?;

        self.set_ignore_nl_option(old_opt);

        Ok(node)
    }

    fn parse_variant_field_internal(
        &mut self,
        variant_field: &mut VariantField,
    ) -> Result<(), Message> {
        let next = self.peek_token();
        match next.lexem {
            Lexem::EndOfLine => {
                *variant_field = VariantField::Common;

                Ok(())
            }

            Lexem::LParen => {
                let location = self.peek_token().location;
                let (ty, info) = self.parse_tuple_def()?;

                if let Type::Tuple(components) = &ty {
                    let mut processed_components: Vec<TypeSpec> = vec![];
                    for ty in components.iter() {
                        processed_components.push(TypeSpec {
                            location,
                            info,
                            ty: ty.clone(),
                        });
                    }
                    *variant_field = VariantField::TupleLike(processed_components);

                    Ok(())
                } else {
                    Err(Message::unexpected_token(next, &[]))
                }
            }

            Lexem::Lcb => {
                let mut node = StructDef::default();
                self.parse_struct_body(&mut node)?;

                *variant_field = VariantField::StructLike(node.fields);

                Ok(())
            }

            _ => Err(Message::new(
                next.location,
                &format!("Unexpected token during parsing enum: {}", next),
            )),
        }
    }
}
