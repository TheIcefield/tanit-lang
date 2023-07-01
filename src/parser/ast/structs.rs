use crate::lexer::TokenType;
use crate::parser::{Id, Parser, ast, put_intent, ast::Ast};

use std::io::Write;

#[derive(Clone)]
pub struct Node {
    pub identifier: Id,
    pub fields:     Vec<ast::variables::Node>,
}

impl ast::IAst for Node {
    fn traverse(&self, stream: &mut ast::Stream, intent: usize) -> std::io::Result<()> {
        writeln!(stream, "{}<struct name=\"{}\">",
            put_intent(intent), self.identifier)?;

        for field in self.fields.iter() {
            field.traverse(stream, intent + 1)?;
        }

        writeln!(stream, "{}</struct>", put_intent(intent))?;
    
        Ok(())
    }
}

pub fn parse_struct_def(parser: &mut Parser) -> Option<Ast> {
    let mut node = parse_header(parser)?;

    node.fields = parse_body_external(parser)?;

    Some(Ast::StructDef { node })
}

pub fn parse_header(parser: &mut Parser) -> Option<Node> {
    parser.consume_token(TokenType::KwStruct)?;

    let identifier = parser.consume_identifier()?;

    Some(Node{
        identifier,
        fields: Vec::<ast::variables::Node>::new()
    })
}

pub fn parse_body_external(parser: &mut Parser) -> Option<Vec<ast::variables::Node>> {
    parser.consume_token(TokenType::Lcb)?;

    let fields = parse_body_internal(parser);

    parser.consume_token(TokenType::Rcb)?;

    fields
}

pub fn parse_body_internal(parser: &mut Parser) -> Option<Vec<ast::variables::Node>> {
    let mut fields = Vec::<ast::variables::Node>::new();
    
    loop {
        let next = parser.peek_token();

        match next.lexem {
            TokenType::Rcb => break,

            TokenType::EndOfLine => {
                parser.get_token();
                continue;
            },

            TokenType::Identifier(_) => {
                let field = ast::variables::parse_param(parser)?;

                fields.push(field);
            },

            _ => {
                parser.error(
                    "Unexpected token when parsing struct fields",
                    next.get_location());

                return None;
            }
        }
    }

    Some(fields)
}


