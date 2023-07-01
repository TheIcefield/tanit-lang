use crate::lexer::TokenType;
use crate::parser::{ast, ast::Ast, Parser};

#[derive(Clone)]
pub struct Node {
    pub identifier: String,
    pub arguments: Vec<Ast>,
}

pub fn parse_call(parser: &mut Parser) -> Option<Vec<Ast>> {
    parser.consume_token(TokenType::LParen)?;

    let mut args = Vec::<Ast>::new();

    loop {
        let next = parser.lexer.peek();

        if next.lexem == TokenType::RParen {
            break;
        }

        let expr = ast::expression_node::parse_expression(parser)?;
        args.push(expr);

        let next = parser.lexer.peek();
        if next.lexem == TokenType::Comma { // continue parsing if ','
            parser.lexer.get();
        } else if next.lexem == TokenType::RParen { // end parsing if ')'
            break;
        } else {
            parser.error("Unexpected token when parsing call",
                         next.get_location());
            return None;
        }
    }

    parser.consume_token(TokenType::LParen)?;

    Some(args)
}
