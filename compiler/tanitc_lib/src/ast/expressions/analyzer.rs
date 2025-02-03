use super::{Expression, ExpressionType};
use crate::analyzer::{symbol_table::SymbolData, Analyze, Analyzer};
use crate::ast::{types::Type, values::ValueType, Ast};
use crate::messages::Message;

use tanitc_lexer::token::Lexem;

impl Analyze for Expression {
    fn get_type(&self, analyzer: &mut Analyzer) -> Type {
        match &self.expr {
            ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } => match operation {
                Lexem::Neq | Lexem::Eq | Lexem::Lt | Lexem::Lte | Lexem::Gt | Lexem::Gte => {
                    Type::Bool
                }

                _ => {
                    let is_conversion = *operation == Lexem::KwAs;

                    let rhs_type = if is_conversion {
                        if let Ast::Type(node) = rhs.as_ref() {
                            node.clone()
                        } else {
                            analyzer
                                .error(Message::new(self.location, "rhs expected to be a type"));
                            Type::new()
                        }
                    } else {
                        rhs.get_type(analyzer)
                    };

                    let mut lhs_type = if let Ast::VariableDef(node) = lhs.as_ref() {
                        node.var_type.clone()
                    } else {
                        lhs.get_type(analyzer)
                    };

                    if let Type::Auto = &mut lhs_type {
                        lhs_type = rhs_type.clone();
                    }

                    if lhs_type == rhs_type {
                        return rhs_type;
                    }

                    if is_conversion {
                        return rhs_type;
                    }

                    analyzer.error(Message::new(
                        self.location,
                        &format!("Mismatched types {:?} and {:?}", lhs_type, rhs_type),
                    ));

                    Type::new()
                }
            },
            ExpressionType::Unary { node, .. } => node.get_type(analyzer),
        }
    }

    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        match &mut self.expr {
            ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } => {
                let is_conversion = *operation == Lexem::KwAs;

                let mut lhs_type = lhs.get_type(analyzer);

                let rhs_type = if is_conversion {
                    if let Ast::Type(node) = rhs.as_ref() {
                        node.clone()
                    } else {
                        unreachable!();
                    }
                } else {
                    rhs.analyze(analyzer)?;
                    rhs.get_type(analyzer)
                };

                if *operation == Lexem::Assign
                    || *operation == Lexem::SubAssign
                    || *operation == Lexem::AddAssign
                    || *operation == Lexem::DivAssign
                    || *operation == Lexem::ModAssign
                    || *operation == Lexem::MulAssign
                    || *operation == Lexem::AndAssign
                    || *operation == Lexem::OrAssign
                    || *operation == Lexem::XorAssign
                    || *operation == Lexem::LShiftAssign
                    || *operation == Lexem::RShiftAssign
                    || is_conversion
                {
                    let does_mutate = !is_conversion;
                    if let Ast::VariableDef(node) = lhs.as_mut() {
                        if analyzer
                            .check_identifier_existance(&node.identifier)
                            .is_ok()
                        {
                            return Err(Message::multiple_ids(
                                node.identifier.location,
                                &node.identifier.get_string(),
                            ));
                        }

                        if Type::Auto == node.var_type {
                            node.var_type = rhs_type.clone();
                            lhs_type = rhs_type.clone();
                        }

                        if node.var_type != rhs_type {
                            analyzer.error(Message::new(self.location,
                                &format!("Variable \"{}\" defined with type \"{:?}\", but is assigned to \"{:?}\"",
                                    node.identifier,
                                    node.var_type,
                                    rhs_type)));
                        }

                        analyzer.add_symbol(
                            &node.identifier,
                            analyzer.create_symbol(SymbolData::VariableDef {
                                var_type: node.var_type.clone(),
                                is_mutable: node.is_mutable,
                                is_initialization: true,
                            }),
                        );
                    } else if let Ast::Value(node) = lhs.as_mut() {
                        match &node.value {
                            ValueType::Identifier(id) => {
                                if let Ok(s) = analyzer.check_identifier_existance(id) {
                                    if let SymbolData::VariableDef { is_mutable, .. } = &s.data {
                                        if !*is_mutable && does_mutate {
                                            analyzer.error(Message::new(
                                                self.location,
                                                &format!(
                                                    "Variable \"{}\" is immutable in current scope",
                                                    id
                                                ),
                                            ));
                                        }
                                    }
                                }
                            }
                            ValueType::Integer(..) | ValueType::Decimal(..) => {}
                            ValueType::Text(..) => analyzer.error(Message::new(
                                self.location,
                                "Cannot perform operation with text in this context",
                            )),
                            ValueType::Array { .. } => analyzer.error(Message::new(
                                self.location,
                                "Cannot perform operation with array in this context",
                            )),
                            ValueType::Tuple { .. } => analyzer.error(Message::new(
                                self.location,
                                "Cannot perform operation with tuple in this context",
                            )),
                            _ => analyzer.error(Message::new(
                                self.location,
                                "Cannot perform operation with this object",
                            )),
                        }
                    } else {
                        lhs.analyze(analyzer)?;
                    }
                } else {
                    lhs.analyze(analyzer)?;
                }

                if lhs_type != rhs_type {
                    if is_conversion {
                        if !lhs_type.is_common() || !rhs_type.is_common() {
                            analyzer.error(Message::new(
                                self.location,
                                &format!("Cannot cast {:?} to {:?}", lhs_type, rhs_type),
                            ));
                        }
                    } else {
                        analyzer.error(Message::new(
                            self.location,
                            &format!("Cannot perform operation with objects with different types: {:?} and {:?}",
                            lhs_type, rhs_type
                        )));
                    }
                }

                Ok(())
            }
            ExpressionType::Unary { node, .. } => node.analyze(analyzer),
        }
    }
}
