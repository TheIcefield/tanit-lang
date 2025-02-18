use super::{Expression, ExpressionType};
use crate::analyzer::{symbol_table::SymbolData, Analyze, Analyzer};
use crate::ast::{values::ValueType, Ast};

use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_ty::Type;

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
                    let mut lhs_type = lhs.get_type(analyzer);

                    if let Type::Auto = &mut lhs_type {
                        lhs_type = rhs.get_type(analyzer);
                    }

                    lhs_type
                }
            },
            ExpressionType::Unary { node, .. } => node.get_type(analyzer),
            ExpressionType::Conversion { ty, .. } => ty.get_type(),
        }
    }

    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        match &mut self.expr {
            ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } => {
                rhs.analyze(analyzer)?;
                let rhs_type = rhs.get_type(analyzer);

                let does_mutate = *operation == Lexem::Assign
                    || *operation == Lexem::SubAssign
                    || *operation == Lexem::AddAssign
                    || *operation == Lexem::DivAssign
                    || *operation == Lexem::ModAssign
                    || *operation == Lexem::MulAssign
                    || *operation == Lexem::AndAssign
                    || *operation == Lexem::OrAssign
                    || *operation == Lexem::XorAssign
                    || *operation == Lexem::LShiftAssign
                    || *operation == Lexem::RShiftAssign;

                if let Ast::VariableDef(node) = lhs.as_mut() {
                    if analyzer.has_symbol(node.identifier) {
                        return Err(Message::multiple_ids(self.location, node.identifier));
                    }

                    if Type::Auto == node.var_type.get_type() {
                        node.var_type.ty = rhs_type.clone();
                    }

                    if node.var_type.get_type() != rhs_type {
                        analyzer.error(Message::new(
                            self.location,
                            &format!("Cannot perform operation on objects with different types: {:?} and {:?}",
                            node.var_type.get_type(), rhs_type
                        )));
                    }

                    analyzer.add_symbol(
                        node.identifier,
                        analyzer.create_symbol(SymbolData::VariableDef {
                            var_type: node.var_type.get_type(),
                            is_mutable: node.is_mutable,
                            is_initialization: true,
                        }),
                    );
                } else if let Ast::Value(node) = lhs.as_mut() {
                    match &node.value {
                        ValueType::Identifier(id) => {
                            if let Some(s) = analyzer.get_first_symbol(*id) {
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
                    let lhs_type = lhs.get_type(analyzer);

                    if lhs_type != rhs_type {
                        analyzer.error(Message::new(
                            self.location,
                            &format!("Cannot perform operation on objects with different types: {:?} and {:?}",
                            lhs_type, rhs_type
                        )));
                    }
                }

                Ok(())
            }
            ExpressionType::Unary { node, .. } => node.analyze(analyzer),
            ExpressionType::Conversion { .. } => todo!(),
        }
    }
}
