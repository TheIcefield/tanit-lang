use super::{VariantDef, VariantField};
use crate::ast::{identifiers::Identifier, structs::StructDef, types::Type, Ast};
use crate::messages::Message;
use crate::parser::{location::Location, token::Lexem, Parser};

use std::collections::HashMap;

impl VariantField {
    pub fn parse(parser: &mut Parser) -> Result<Self, Message> {
        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);
        let res = Self::parse_internal(parser);
        parser.set_ignore_nl_option(old_opt);

        res
    }

    fn parse_internal(parser: &mut Parser) -> Result<Self, Message> {
        let next = parser.peek_token();
        match next.lexem {
            Lexem::EndOfLine => Ok(VariantField::Common),

            Lexem::LParen => {
                if let Type::Tuple { components } = Type::parse_tuple_def(parser)? {
                    Ok(Self::TupleLike(components))
                } else {
                    Err(Message::unexpected_token(next, &[]))
                }
            }

            Lexem::Lcb => {
                if let Ast::StructDef { node } = StructDef::parse_body_external(parser)? {
                    if !node.internals.is_empty() {
                        parser.error(Message::new(
                            next.location,
                            "Internal structs are not allowed here",
                        ));
                    }

                    return Ok(VariantField::StructLike(node.fields));
                }
                unreachable!()
            }

            _ => {
                parser.error(Message::new(
                    next.location,
                    &format!("Unexpected token during parsing enum: {}", next),
                ));
                unreachable!()
            }
        }
    }
}

impl VariantDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let identifier = Self::parse_header(parser)?.identifier;

        if let Ast::VariantDef { mut node } = Self::parse_body(parser)? {
            node.identifier = identifier;
            return Ok(Ast::VariantDef { node });
        }

        unreachable!()
    }

    fn parse_header(parser: &mut Parser) -> Result<Self, Message> {
        let location = parser.consume_token(Lexem::KwVariant)?.location;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        Ok(VariantDef {
            location,
            identifier,
            fields: HashMap::new(),
            internals: Vec::new(),
        })
    }

    fn parse_body(parser: &mut Parser) -> Result<Ast, Message> {
        parser.consume_token(Lexem::Lcb)?;
        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);
        let fields = Self::parse_body_internal(parser);
        parser.set_ignore_nl_option(old_opt);

        parser.consume_token(Lexem::Rcb)?;

        fields
    }

    fn parse_body_internal(parser: &mut Parser) -> Result<Ast, Message> {
        let mut fields = HashMap::<Identifier, VariantField>::new();
        let mut internals = Vec::<Ast>::new();

        loop {
            let next = parser.peek_token();

            match &next.lexem {
                Lexem::Rcb => break,

                Lexem::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                Lexem::KwStruct => internals.push(StructDef::parse(parser)?),

                Lexem::KwVariant => internals.push(VariantDef::parse(parser)?),

                Lexem::Identifier(id) => {
                    let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

                    if fields.contains_key(&identifier) {
                        parser.error(Message::new(
                            next.location,
                            &format!("Enum has already field with identifier \"{}\"", id),
                        ));
                        continue;
                    }

                    fields.insert(identifier, VariantField::parse(parser)?);

                    parser.consume_new_line()?;
                }

                Lexem::Lcb => {
                    return Err(Message::new(
                        next.location,
                        &format!(
                            "{}\nHelp: {}{}",
                            "Unexpected token: \"{\" during parsing enum fields.",
                            "If you tried to declare struct-like field, place \"{\" ",
                            "in the same line with name of the field."
                        ),
                    ));
                }

                _ => {
                    return Err(Message::unexpected_token(next, &[]));
                }
            }
        }

        Ok(Ast::VariantDef {
            node: Self {
                location: Location::new(),
                identifier: Identifier::new(),
                fields,
                internals,
            },
        })
    }
}
