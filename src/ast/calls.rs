use crate::ast::{expressions, Ast, IAst};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Parser};

use std::io::Write;

#[derive(Clone)]
pub struct Node {
    pub identifier: String,
    pub arguments: Vec<Ast>,
}

impl IAst for Node {
    fn traverse(&self, stream: &mut super::Stream, intent: usize) -> std::io::Result<()> {
        writeln!(
            stream,
            "{}<call name=\"{}\">",
            put_intent(intent),
            self.identifier
        )?;

        for arg in self.arguments.iter() {
            arg.traverse(stream, intent + 1)?
        }

        writeln!(stream, "{}</call>", put_intent(intent))?;

        Ok(())
    }
}

pub fn parse_call(parser: &mut Parser) -> Option<Vec<Ast>> {
    parser.consume_token(TokenType::LParen)?;

    let mut args = Vec::<Ast>::new();

    loop {
        let next = parser.peek_token();

        if next.lexem == TokenType::RParen {
            break;
        }

        let expr = expressions::parse_expression(parser)?;
        args.push(expr);

        let next = parser.peek_token();
        if next.lexem == TokenType::Comma {
            // continue parsing if ','
            parser.get_token();
            continue;
        } else if next.lexem == TokenType::RParen {
            // end parsing if ')'
            break;
        } else {
            parser.error("Unexpected token when parsing call", next.get_location());
            return None;
        }
    }

    parser.consume_token(TokenType::RParen)?;

    Some(args)
}
