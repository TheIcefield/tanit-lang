use super::ModuleDef;
use crate::ast::{scopes::Scope, Ast};
use crate::unit::{self, Unit};

use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_parser::Parser;

impl ModuleDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::default();

        node.parse_header(parser)?;
        node.parse_body(parser)?;

        Ok(Ast::from(node))
    }

    fn parse_header(&mut self, parser: &mut Parser) -> Result<(), Message> {
        let next = parser.peek_token();
        self.location = next.location;

        if Lexem::KwDef == next.lexem {
            parser.consume_token(Lexem::KwDef)?;
            self.is_external = true;
        }

        parser.consume_token(Lexem::KwModule)?;

        self.identifier = parser.consume_identifier()?;

        Ok(())
    }

    fn parse_body(&mut self, parser: &mut Parser) -> Result<(), Message> {
        if self.is_external {
            self.parse_external_body(parser)?;
        } else {
            self.parse_internal_body(parser)?;
        }

        Ok(())
    }

    fn parse_internal_body(&mut self, parser: &mut Parser) -> Result<(), Message> {
        parser.consume_token(Lexem::Lcb)?;

        let scope = Scope::parse_global(parser)?;

        parser.consume_token(Lexem::Rcb)?;

        if let Ast::Scope(node) = scope {
            self.body = Some(node);
        } else {
            return Err(Message::unreachable(self.location));
        }

        Ok(())
    }

    fn parse_external_body(&mut self, parser: &mut Parser) -> Result<(), Message> {
        let name: String = self.identifier.into();

        let mut path = parser
            .get_path()
            .chars()
            .rev()
            .collect::<String>()
            .splitn(2, '/')
            .collect::<Vec<&str>>()[1]
            .chars()
            .rev()
            .collect::<String>();

        path.push('/');
        path.push_str(&name);

        let path = name.clone();

        let mut unit_exists: bool;

        {
            let mut path = path.clone();
            path.push_str(".tt");

            unit_exists = std::path::Path::new(&path).exists();
            if unit_exists {
                unit::register_unit(
                    Unit::builder()
                        .set_name(name.clone())
                        .set_path(path)
                        .build(),
                );
            }
        }

        if !unit_exists {
            let mut path = path.clone();
            path.push_str("/mod.tt");

            unit_exists = std::path::Path::new(&path).exists();
            if unit_exists {
                unit::register_unit(
                    Unit::builder()
                        .set_name(name.clone())
                        .set_path(path)
                        .build(),
                );
            }
        }

        Ok(())
    }
}
