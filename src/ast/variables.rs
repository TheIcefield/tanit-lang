use crate::analyzer::SymbolData;
use crate::ast::{
    expressions::Expression, identifiers::Identifier, types::Type, Ast, IAst, Stream,
};
use crate::codegen::CodeGenStream;
use crate::error_listener::{
    MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR, UNEXPECTED_TOKEN_ERROR_STR,
    VARIABLE_DEFINED_WITHOUT_TYPE_ERROR_STR,
};
use crate::lexer::Lexem;
use crate::parser::{put_intent, Parser};

use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct VariableNode {
    pub identifier: Identifier,
    pub var_type: Type,
    pub is_global: bool,
    pub is_mutable: bool,
}

impl VariableNode {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, &'static str> {
        let next = parser.peek_token();

        let is_global = match next.lexem {
            Lexem::KwLet => {
                parser.get_token();
                false
            }

            Lexem::KwStatic => {
                parser.get_token();
                true
            }

            _ => {
                parser.error(
                    "Unexpected token. There only \"let\", \"static\", allowed",
                    next.location,
                );
                return Err(UNEXPECTED_TOKEN_ERROR_STR);
            }
        };

        let next = parser.peek_token();
        let is_mutable = match next.lexem {
            Lexem::KwMut => {
                parser.get_token();
                true
            }

            Lexem::KwConst => {
                parser.get_token();
                false
            }

            _ => false,
        };

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        let next = parser.peek_token();

        let mut var_type: Option<Type> = None;
        let mut rvalue: Option<Ast> = None;

        if Lexem::Colon == next.lexem {
            parser.consume_token(Lexem::Colon)?;

            var_type = Some(Type::parse(parser)?);
        }

        let next = parser.peek_token();

        if Lexem::Assign == next.lexem {
            parser.get_token();

            rvalue = Some(Expression::parse(parser)?);
        }

        if var_type.is_none() && rvalue.is_none() {
            parser.error(
                &format!(
                    "Variable {} defined without type. Need to specify type or use with rvalue",
                    identifier
                ),
                next.location,
            );
            return Err(VARIABLE_DEFINED_WITHOUT_TYPE_ERROR_STR);
        }

        if var_type.is_none() && is_global {
            parser.error(
                &format!(
                    "Variable {} defined without type, but marked as static. Need to specify type",
                    identifier
                ),
                next.location,
            );
            return Err(VARIABLE_DEFINED_WITHOUT_TYPE_ERROR_STR);
        }

        if var_type.is_none() && rvalue.is_some() {
            var_type = Some(Type::Custom("@auto".to_string()));
        }

        let var_node = Ast::VariableDef {
            node: Self {
                identifier,
                var_type: var_type.unwrap(),
                is_global,
                is_mutable,
            },
        };

        if let Some(rhs) = rvalue {
            return Ok(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation: Lexem::Assign,
                    lhs: Box::new(var_node),
                    rhs: Box::new(rhs),
                }),
            });
        }

        Ok(var_node)
    }

    /* parse function param */
    pub fn parse_param(parser: &mut Parser) -> Result<Self, &'static str> {
        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        parser.consume_token(Lexem::Colon)?;

        let var_type = Type::parse(parser)?;

        Ok(Self {
            identifier,
            var_type,
            is_global: false,
            is_mutable: true,
        })
    }
}

impl IAst for VariableNode {
    fn get_type(&self, _analyzer: &mut crate::analyzer::Analyzer) -> Type {
        self.var_type.clone()
    }

    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        if analyzer
            .check_identifier_existance(&self.identifier)
            .is_ok()
        {
            analyzer.error(&format!(
                "Identifier \"{}\" defined multiple times",
                &self.identifier
            ));
            return Err(MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR);
        }

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::VariableDef {
                var_type: self.var_type.clone(),
                is_mutable: self.is_mutable,
                is_initialization: false,
            }),
        );

        Ok(())
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(
            stream,
            "{}<variable name=\"{}\" is_global=\"{}\" is_mutable=\"{}\">",
            put_intent(intent),
            self.identifier,
            self.is_global,
            self.is_mutable
        )?;

        self.var_type.traverse(stream, intent + 1)?;

        writeln!(stream, "{}</variable>", put_intent(intent))?;

        Ok(())
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        self.var_type.codegen(stream)?;

        write!(stream, "{}", if self.is_mutable { " " } else { " const " })?;

        self.identifier.codegen(stream)?;

        Ok(())
    }
}
