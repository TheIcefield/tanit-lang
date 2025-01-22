use super::StructDef;
use crate::ast::{identifiers::Identifier, types::Type, variants::VariantDef, Ast};
use crate::messages::Message;
use crate::parser::{location::Location, token::Lexem, Parser};

use std::collections::BTreeMap;

impl StructDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let identifier = Self::parse_header(parser)?.identifier;

        if let Ast::StructDef { mut node } = Self::parse_body_external(parser)? {
            node.identifier = identifier;
            return Ok(Ast::StructDef { node });
        }

        unreachable!()
    }

    pub fn parse_header(parser: &mut Parser) -> Result<Self, Message> {
        let location = parser.consume_token(Lexem::KwStruct)?.location;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        Ok(StructDef {
            location,
            identifier,
            fields: BTreeMap::new(),
            internals: Vec::new(),
        })
    }

    pub fn parse_body_external(parser: &mut Parser) -> Result<Ast, Message> {
        parser.consume_token(Lexem::Lcb)?;

        let fields = Self::parse_body_internal(parser);

        parser.consume_token(Lexem::Rcb)?;

        fields
    }

    pub fn parse_body_internal(parser: &mut Parser) -> Result<Ast, Message> {
        let mut fields = BTreeMap::<Identifier, Type>::new();
        let mut internals = Vec::<Ast>::new();

        loop {
            let next = parser.peek_token();

            match &next.lexem {
                Lexem::Rcb => break,

                Lexem::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                Lexem::KwStruct => {
                    internals.push(StructDef::parse(parser)?);
                }

                Lexem::KwVariant => {
                    internals.push(VariantDef::parse(parser)?);
                }

                Lexem::Identifier(id) => {
                    let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

                    if fields.contains_key(&identifier) {
                        parser.error(Message::new(
                            next.location,
                            &format!("Struct has already field with identifier {}", id),
                        ));
                        continue;
                    }

                    parser.consume_token(Lexem::Colon)?;

                    fields.insert(identifier, Type::parse(parser)?);
                }

                _ => {
                    return Err(Message::new(
                        next.location,
                        "Unexpected token when parsing struct fields",
                    ));
                }
            }
        }

        Ok(Ast::StructDef {
            node: Self {
                location: Location::new(),
                identifier: Identifier::new(),
                fields,
                internals,
            },
        })
    }
}
