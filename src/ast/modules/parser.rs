use super::ModuleDef;
use crate::ast::{
    identifiers::{Identifier, IdentifierType},
    scopes::Scope,
    Ast,
};
use crate::messages::{self, Message};
use crate::parser::{lexer::Lexer, token::Lexem, Parser};

impl ModuleDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::default();

        node.parse_header(parser)?;
        node.parse_body(parser)?;

        Ok(Ast::ModuleDef { node })
    }

    fn parse_header(&mut self, parser: &mut Parser) -> Result<(), Message> {
        self.location = parser.consume_token(Lexem::KwModule)?.location;
        self.identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        Ok(())
    }

    fn parse_body(&mut self, parser: &mut Parser) -> Result<(), Message> {
        self.body = Some(Box::new(Scope::parse_global(parser)?));

        Ok(())
    }

    pub fn parse_external(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::default();

        node.parse_header(parser)?;
        node.parse_external_body(parser)?;

        Ok(Ast::ModuleDef { node })
    }

    fn parse_external_body(&mut self, parser: &mut Parser) -> Result<(), Message> {
        let identifier = match &self.identifier.identifier {
            IdentifierType::Common(id) => id.clone(),
            IdentifierType::Complex(..) => unimplemented!(),
        };

        let mut path = parser.get_path();
        let verbose = parser.is_token_verbose();
        let mut body: Option<Box<Ast>> = None;

        path = path
            .chars()
            .rev()
            .collect::<String>()
            .splitn(2, '/')
            .collect::<Vec<&str>>()[1]
            .chars()
            .rev()
            .collect::<String>();

        path.push('/');
        path.push_str(&identifier);

        {
            let mut path = path.clone();
            path.push_str(".tt");

            let lexer = Lexer::from_file(&path, verbose);

            if let Ok(lexer) = lexer {
                let mut parser_int = Parser::new(lexer);

                let parse_res = parser_int.parse();

                if parser_int.has_errors() {
                    messages::print_messages(&parser_int.get_errors());
                } else if parser_int.has_warnings() {
                    messages::print_messages(&parser_int.get_warnings());
                }

                match parse_res {
                    None => {
                        return Err(Message::new(
                            parser.get_location(),
                            &format!(
                                "Error occured during parsing module \"{}\" body",
                                identifier
                            ),
                        ));
                    }

                    Some(node) => {
                        body = Some(Box::new(node));
                    }
                }
            }
        }

        if body.is_none() {
            let mut path = path.clone();
            path.push_str("/mod.tt");

            // println!("Another try parse {}", path);

            let lexer = Lexer::from_file(&path, verbose);

            if let Ok(lexer) = lexer {
                let mut parser_int = Parser::new(lexer);

                let parser_res = parser_int.parse();

                if parser_int.has_errors() {
                    messages::print_messages(&parser_int.get_errors());
                } else if parser_int.has_warnings() {
                    messages::print_messages(&parser_int.get_warnings());
                }

                match parser_res {
                    None => {
                        return Err(Message::new(
                            parser.get_location(),
                            &format!(
                                "Error occured during parsing module \"{}\" body",
                                identifier
                            ),
                        ));
                    }

                    Some(ast) => {
                        body = Some(Box::new(ast));
                    }
                }
            }
        }

        if body.is_none() {
            return Err(Message::new(
                parser.get_location(),
                &format!("Not found definition for module \"{}\"", identifier),
            ));
        }

        self.body = body;

        Ok(())
    }
}
