use crate::ast::{scopes, types, variables, Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Id, Parser};

use std::io::Write;

#[derive(Clone)]
pub struct Node {
    pub identifier: Id,
    pub return_type: types::Type,
    pub parameters: Vec<variables::Node>,
    pub body: Option<Box<Ast>>,
    pub is_static: bool,
}

impl IAst for Node {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(
            stream,
            "{}<function name=\"{}\" is_static=\"{}\">",
            put_intent(intent),
            self.identifier,
            self.is_static
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

pub fn parse_func_def(parser: &mut Parser) -> Option<Ast> {
    let mut node = parse_header(parser)?;

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
            node.body = Some(Box::new(parse_body(parser)?));
        }

        _ => {
            parser.error(
                &format!("Unexpected token \"{}\", allowed EOL or \'}}\'", next),
                next.get_location(),
            );
            return None;
        }
    }

    Some(Ast::FuncDef { node })
}

pub fn parse_header(parser: &mut Parser) -> Option<Node> {
    let mut is_static = false;

    let next = parser.peek_token();
    if next.lexem == TokenType::KwStatic {
        is_static = true;
        parser.get_token();
    }

    parser.consume_token(TokenType::KwFunc)?;

    let identifier = parser.consume_identifier()?;

    let parameters = parse_header_params(parser)?;

    let next = parser.peek_token();
    let return_type = if next.lexem == TokenType::Arrow {
        parser.get_token();
        types::Type::parse(parser)?
    } else {
        types::Type::Tuple {
            components: Vec::<types::Type>::new(),
        }
    };

    Some(Node {
        identifier,
        return_type,
        parameters,
        body: None,
        is_static,
    })
}

pub fn parse_header_params(parser: &mut Parser) -> Option<Vec<variables::Node>> {
    parser.consume_token(TokenType::LParen)?;

    let mut parameters = Vec::<variables::Node>::new();
    loop {
        let next = parser.peek_token();

        if next.is_identifier() {
            let param = variables::parse_param(parser)?;
            parameters.push(param);

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
                return None;
            }
        } else if next.lexem == TokenType::RParen {
            parser.get_token();
            break;
        } else {
            parser.error(
                &format!("Unexpected token \"{}\", allowed identifier or \')\'", next),
                next.get_location(),
            );
            return None;
        }
    }

    Some(parameters)
}

pub fn parse_body(parser: &mut Parser) -> Option<Ast> {
    scopes::parse_local_external(parser)
}
