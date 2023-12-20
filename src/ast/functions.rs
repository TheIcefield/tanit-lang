use crate::analyzer::SymbolData;
use crate::ast::{scopes, types, variables::VariableNode, Ast, IAst, Stream};
use crate::error_listener::{MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR, UNEXPECTED_TOKEN_ERROR_STR};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Id, Parser};

use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct FunctionNode {
    pub identifier: Id,
    pub return_type: types::Type,
    pub parameters: Vec<Ast>,
    pub body: Option<Box<Ast>>,
}

impl FunctionNode {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, &'static str> {
        let mut node = Self::parse_header(parser)?;

        loop {
            let tkn = parser.peek_token();

            if tkn.lexem == TokenType::EndOfLine {
                parser.get_token();
            } else {
                break;
            }
        }

        let next = parser.peek_token();
        match next.lexem {
            TokenType::Lcb => {
                node.body = Some(Box::new(scopes::Scope::parse_local(parser)?));
            }

            _ => {
                parser.error(
                    &format!("Unexpected token \"{}\", allowed EOL or \'}}\'", next),
                    next.get_location(),
                );
                return Err(UNEXPECTED_TOKEN_ERROR_STR);
            }
        }

        Ok(Ast::FuncDef { node })
    }

    pub fn parse_header(parser: &mut Parser) -> Result<Self, &'static str> {
        parser.consume_token(TokenType::KwFunc)?;

        let identifier = parser.consume_identifier()?;

        let parameters = Self::parse_header_params(parser)?;

        let next = parser.peek_token();
        let return_type = if next.lexem == TokenType::Arrow {
            parser.get_token();
            types::Type::parse(parser)?
        } else {
            types::Type::Tuple {
                components: Vec::<types::Type>::new(),
            }
        };

        Ok(Self {
            identifier,
            return_type,
            parameters,
            body: None,
        })
    }

    pub fn parse_header_params(parser: &mut Parser) -> Result<Vec<Ast>, &'static str> {
        parser.consume_token(TokenType::LParen)?;

        let mut parameters = Vec::<Ast>::new();
        loop {
            let next = parser.peek_token();

            if next.is_identifier() {
                parameters.push(Ast::VariableDef {
                    node: VariableNode::parse_param(parser)?,
                });

                let next = parser.peek_token();
                if next.lexem == TokenType::Comma {
                    parser.get_token();
                } else if next.lexem == TokenType::RParen {
                    continue;
                } else {
                    parser.error(
                        &format!("Unexpected token \"{}\", allowed \',\' or \')\'", next),
                        next.get_location(),
                    );
                    return Err(UNEXPECTED_TOKEN_ERROR_STR);
                }
            } else if next.lexem == TokenType::RParen {
                parser.get_token();
                break;
            } else {
                parser.error(
                    &format!("Unexpected token \"{}\", allowed identifier or \')\'", next),
                    next.get_location(),
                );
                return Err(UNEXPECTED_TOKEN_ERROR_STR);
            }
        }

        Ok(parameters)
    }
}

impl IAst for FunctionNode {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        if let Ok(_ss) = analyzer.check_identifier_existance(&self.identifier) {
            analyzer.error(&format!(
                "Identifier \"{}\" defined multiple times",
                &self.identifier
            ));
            return Err(MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR);
        }

        analyzer.scope.push(&self.identifier);

        let mut arguments = Vec::<types::Type>::new();
        for p in self.parameters.iter_mut() {
            if let Ast::VariableDef { node } = p {
                arguments.push(node.var_type.clone());
                p.analyze(analyzer)?;
            }
        }

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::FunctionDef {
                args: arguments.clone(),
                return_type: self.return_type.clone(),
                is_declaration: self.body.is_some(),
            }),
        );

        if let Some(body) = &mut self.body {
            body.analyze(analyzer)?;
        }

        analyzer.scope.pop();

        Ok(())
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(
            stream,
            "{}<function name=\"{}\">",
            put_intent(intent),
            self.identifier,
        )?;

        writeln!(stream, "{}<return-type>", put_intent(intent + 1))?;

        self.return_type.traverse(stream, intent + 2)?;

        writeln!(stream, "{}</return-type>", put_intent(intent + 1))?;

        writeln!(stream, "{}<parameters>", put_intent(intent + 1))?;

        for param in self.parameters.iter() {
            param.traverse(stream, intent + 2)?;
        }

        writeln!(stream, "{}</parameters>", put_intent(intent + 1))?;

        match &self.body {
            Some(node) => {
                writeln!(stream, "{}<body>", put_intent(intent + 1))?;

                node.traverse(stream, intent + 2)?;

                writeln!(stream, "{}</body>", put_intent(intent + 1))?;
            }

            None => {}
        }

        writeln!(stream, "{}</function>", put_intent(intent))?;

        Ok(())
    }
}
