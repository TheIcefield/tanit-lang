use crate::ast::{Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Id, Parser};

use std::io::Write;

#[derive(Clone)]
pub struct Type {
    pub identifier: Id,
    pub children: Vec<Type>,
}

#[derive(Clone)]
pub struct Alias {
    pub identifier: Id,
    pub value: Type,
}

impl IAst for Type {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        if self.children.is_empty() {
            writeln!(
                stream,
                "{}<type name=\"{}\"/>",
                put_intent(intent),
                self.identifier
            )?;
            return Ok(());
        }

        writeln!(
            stream,
            "{}<type name=\"{}\">",
            put_intent(intent),
            self.identifier
        )?;

        for i in self.children.iter() {
            i.traverse(stream, intent + 1)?;
        }

        writeln!(stream, "{}</type>", put_intent(intent))?;

        Ok(())
    }
}

impl IAst for Alias {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(
            stream,
            "{}<alias name=\"{}\">",
            put_intent(intent),
            self.identifier
        )?;

        self.value.traverse(stream, intent + 1)?;

        writeln!(stream, "{}</alias>", put_intent(intent))?;

        Ok(())
    }
}

pub fn parse_type(parser: &mut Parser) -> Option<Type> {
    let identifier = parser.consume_identifier()?;
    let mut children = Vec::<Type>::new();

    if parser.peek_singular().lexem == TokenType::Lt {
        children = parse_template_args(parser)?;
    }

    Some(Type {
        identifier,
        children,
    })
}

pub fn parse_template_args(parser: &mut Parser) -> Option<Vec<Type>> {
    parser.consume_token(TokenType::Lt)?;

    let mut children = Vec::<Type>::new();
    loop {
        let child = parse_type(parser)?;
        children.push(child);

        let next = parser.peek_singular();
        if next.lexem == TokenType::Gt {
            break;
        } else {
            parser.consume_token(TokenType::Comma)?;
        }
    }

    parser.get_singular();

    Some(children)
}

pub fn parse_alias_def(parser: &mut Parser) -> Option<Ast> {
    parser.consume_token(TokenType::KwAlias)?;

    let identifier = parser.consume_identifier()?;

    parser.consume_token(TokenType::Assign)?;

    let value = parse_type(parser)?;

    Some(Ast::AliasDef {
        node: Alias { identifier, value },
    })
}
