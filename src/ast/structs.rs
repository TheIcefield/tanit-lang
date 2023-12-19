use crate::ast::{types, Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Id, Parser};

use std::collections::HashMap;
use std::io::Write;

#[derive(Clone)]
pub struct StructNode {
    pub identifier: Id,
    pub fields: HashMap<Id, types::Type>,
    pub internals: Vec<Ast>,
}

impl StructNode {
    pub fn parse_def(parser: &mut Parser) -> Option<Ast> {
        let identifier = Self::parse_header(parser)?.identifier;

        if let Ast::StructDef { mut node } = Self::parse_body_external(parser)? {
            node.identifier = identifier;
            return Some(Ast::StructDef { node });
        }

        None
    }

    pub fn parse_header(parser: &mut Parser) -> Option<StructNode> {
        parser.consume_token(TokenType::KwStruct)?;

        let identifier = parser.consume_identifier()?;

        Some(StructNode {
            identifier,
            fields: HashMap::new(),
            internals: Vec::new(),
        })
    }

    pub fn parse_body_external(parser: &mut Parser) -> Option<Ast> {
        parser.consume_token(TokenType::Lcb)?;

        let fields = Self::parse_body_internal(parser);

        parser.consume_token(TokenType::Rcb)?;

        fields
    }

    pub fn parse_body_internal(parser: &mut Parser) -> Option<Ast> {
        let mut fields = HashMap::<Id, types::Type>::new();
        let mut internals = Vec::<Ast>::new();

        loop {
            let next = parser.peek_token();

            match &next.lexem {
                TokenType::Rcb => break,

                TokenType::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                TokenType::KwStruct => {
                    internals.push(StructNode::parse_def(parser)?);
                }

                TokenType::KwEnum => {
                    internals.push(EnumNode::parse_def(parser)?);
                }

                TokenType::Identifier(id) => {
                    parser.consume_identifier()?;

                    if fields.contains_key(id) {
                        parser.error(
                            "Struct has already field with the same identifier",
                            next.get_location(),
                        );
                        continue;
                    }

                    parser.consume_token(TokenType::Colon)?;

                    fields.insert(id.clone(), types::Type::parse(parser)?);
                }

                _ => {
                    parser.error(
                        "Unexpected token when parsing struct fields",
                        next.get_location(),
                    );

                    return None;
                }
            }
        }

        Some(Ast::StructDef {
            node: Self {
                identifier: String::new(),
                fields,
                internals,
            },
        })
    }
}

impl IAst for StructNode {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(
            stream,
            "{}<struct-def name=\"{}\">",
            put_intent(intent),
            self.identifier
        )?;

        for internal in self.internals.iter() {
            internal.traverse(stream, intent + 1)?;
        }

        for field in self.fields.iter() {
            writeln!(
                stream,
                "{}<field name=\"{}\">",
                put_intent(intent + 1),
                field.0
            )?;

            field.1.traverse(stream, intent + 2)?;

            writeln!(stream, "{}</field>", put_intent(intent + 1))?;
        }

        writeln!(stream, "{}</struct-def>", put_intent(intent))?;

        Ok(())
    }
}

#[derive(Clone)]
pub enum EnumField {
    StructLike(HashMap<Id, types::Type>),
    TupleLike(Vec<types::Type>),
    Common,
}

impl EnumField {
    pub fn parse(parser: &mut Parser) -> Option<EnumField> {
        let next = parser.peek_token();
        match next.lexem {
            TokenType::EndOfLine => Some(EnumField::Common),
            TokenType::LParen => {
                if let types::Type::Tuple { components } = types::Type::parse_tuple_def(parser)? {
                    Some(Self::TupleLike(components))
                } else {
                    None
                }
            }
            TokenType::Lcb => {
                if let Ast::StructDef { node } = StructNode::parse_body_external(parser)? {
                    if !node.internals.is_empty() {
                        parser.error("Internal structs are not allowed here", next.get_location());
                    }

                    return Some(EnumField::StructLike(node.fields));
                }
                None
            }
            _ => {
                parser.error(
                    &format!("Unexpected token during parsing enum: {}", next),
                    next.get_location(),
                );
                None
            }
        }
    }
}

impl IAst for EnumField {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match self {
            Self::StructLike(s) => {
                for f in s.iter() {
                    writeln!(stream, "{}<field name=\"{}\">", put_intent(intent), f.0)?;

                    f.1.traverse(stream, intent + 1)?;

                    writeln!(stream, "{}</field>", put_intent(intent))?;
                }
            }
            Self::TupleLike(t) => {
                for c in t.iter() {
                    c.traverse(stream, intent)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct EnumNode {
    pub identifier: Id,
    pub fields: HashMap<Id, EnumField>,
    pub internals: Vec<Ast>,
}

impl EnumNode {
    pub fn parse_def(parser: &mut Parser) -> Option<Ast> {
        let identifier = Self::parse_header(parser)?.identifier;

        if let Ast::EnumDef { mut node } = Self::parse_body_external(parser)? {
            node.identifier = identifier;
            return Some(Ast::EnumDef { node });
        }

        None
    }

    pub fn parse_header(parser: &mut Parser) -> Option<EnumNode> {
        parser.consume_token(TokenType::KwEnum)?;

        let identifier = parser.consume_identifier()?;

        Some(EnumNode {
            identifier,
            fields: HashMap::new(),
            internals: Vec::new(),
        })
    }

    pub fn parse_body_external(parser: &mut Parser) -> Option<Ast> {
        parser.consume_token(TokenType::Lcb)?;

        let fields = Self::parse_body_internal(parser);

        parser.consume_token(TokenType::Rcb)?;

        fields
    }

    pub fn parse_body_internal(parser: &mut Parser) -> Option<Ast> {
        let mut fields = HashMap::<Id, EnumField>::new();
        let mut internals = Vec::<Ast>::new();

        loop {
            let next = parser.peek_token();

            match &next.lexem {
                TokenType::Rcb => break,

                TokenType::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                TokenType::KwStruct => {
                    internals.push(StructNode::parse_def(parser)?);
                }

                TokenType::KwEnum => {
                    internals.push(EnumNode::parse_def(parser)?);
                }

                TokenType::Identifier(id) => {
                    parser.consume_identifier()?;

                    if fields.contains_key(id) {
                        parser.error(
                            "Enum has already field with the same identifier",
                            next.get_location(),
                        );
                        continue;
                    }

                    fields.insert(id.clone(), EnumField::parse(parser)?);

                    parser.consume_new_line()?;
                }

                _ => {
                    parser.error(
                        "Unexpected token when parsing enum fields",
                        next.get_location(),
                    );

                    return None;
                }
            }
        }

        Some(Ast::EnumDef {
            node: Self {
                identifier: String::new(),
                fields,
                internals,
            },
        })
    }
}

impl IAst for EnumNode {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(
            stream,
            "{}<enum-def name=\"{}\">",
            put_intent(intent),
            self.identifier
        )?;

        for internal in self.internals.iter() {
            internal.traverse(stream, intent + 1)?;
        }

        for field in self.fields.iter() {
            if matches!(field.1, EnumField::Common) {
                writeln!(
                    stream,
                    "{}<field name=\"{}\"/>",
                    put_intent(intent + 1),
                    field.0
                )?;
                continue;
            }

            writeln!(
                stream,
                "{}<field name=\"{}\">",
                put_intent(intent + 1),
                field.0
            )?;

            field.1.traverse(stream, intent + 2)?;

            writeln!(stream, "{}</field>", put_intent(intent + 1))?;
        }

        writeln!(stream, "{}</enum-def>", put_intent(intent))?;

        Ok(())
    }
}
