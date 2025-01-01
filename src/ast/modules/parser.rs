use super::ModuleDef;
use crate::ast::{
    identifiers::{Identifier, IdentifierType},
    scopes::Scope,
    Ast,
};
use crate::messages::Message;
use crate::parser::{lexer::Lexer, token::Lexem, Parser};

impl ModuleDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::parse_header(parser)?;

        node.body = Box::new(Scope::parse_global(parser)?);

        Ok(Ast::ModuleDef { node })
    }

    fn parse_header(parser: &mut Parser) -> Result<Self, Message> {
        let location = parser.consume_token(Lexem::KwModule)?.location;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        Ok(Self {
            location,
            identifier,
            body: Box::new(Ast::Scope {
                node: Scope {
                    statements: Vec::new(),
                    is_global: true,
                },
            }),
        })
    }

    pub fn parse_ext_module(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::parse_header(parser)?;

        node.body = Self::parse_ext_body(&node.identifier, parser)?;

        Ok(Ast::ModuleDef { node })
    }

    fn parse_ext_body(identifier: &Identifier, parser: &mut Parser) -> Result<Box<Ast>, Message> {
        let identifier = match &identifier.identifier {
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

                match parser_int.parse() {
                    Err(messages) => {
                        for err in messages.0.iter() {
                            parser.error(err.clone())
                        }
                        for warn in messages.1.iter() {
                            parser.warning(warn.clone())
                        }
                        return Err(Message::new(
                            parser.get_location(),
                            &format!(
                                "Error occured during parsing module \"{}\" body",
                                identifier
                            ),
                        ));
                    }

                    Ok(node) => {
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

                match parser_int.parse() {
                    Err(messages) => {
                        for err in messages.0.iter() {
                            parser.error(err.clone())
                        }
                        for warn in messages.1.iter() {
                            parser.warning(warn.clone())
                        }
                        return Err(Message::new(
                            parser.get_location(),
                            &format!(
                                "Error occured during parsing module \"{}\" body",
                                identifier
                            ),
                        ));
                    }

                    Ok(node) => {
                        body = Some(Box::new(node));
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

        Ok(body.unwrap())
    }
}
