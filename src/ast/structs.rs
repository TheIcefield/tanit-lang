use crate::analyzer::SymbolData;
use crate::ast::{types, Ast, IAst, Stream};
use crate::error_listener::{
    MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR, UNEXPECTED_NODE_PARSED_ERROR_STR,
    UNEXPECTED_TOKEN_ERROR_STR,
};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Id, Parser};

use std::collections::HashMap;
use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct StructNode {
    pub identifier: Id,
    pub fields: HashMap<Id, types::Type>,
    pub internals: Vec<Ast>,
}

impl StructNode {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, &'static str> {
        let identifier = Self::parse_header(parser)?.identifier;

        if let Ast::StructDef { mut node } = Self::parse_body_external(parser)? {
            node.identifier = identifier;
            return Ok(Ast::StructDef { node });
        }

        Err(UNEXPECTED_NODE_PARSED_ERROR_STR)
    }

    pub fn parse_header(parser: &mut Parser) -> Result<Self, &'static str> {
        parser.consume_token(TokenType::KwStruct)?;

        let identifier = parser.consume_identifier()?;

        Ok(StructNode {
            identifier,
            fields: HashMap::new(),
            internals: Vec::new(),
        })
    }

    pub fn parse_body_external(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(TokenType::Lcb)?;

        let fields = Self::parse_body_internal(parser);

        parser.consume_token(TokenType::Rcb)?;

        fields
    }

    pub fn parse_body_internal(parser: &mut Parser) -> Result<Ast, &'static str> {
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

                    return Err(UNEXPECTED_TOKEN_ERROR_STR);
                }
            }
        }

        Ok(Ast::StructDef {
            node: Self {
                identifier: String::new(),
                fields,
                internals,
            },
        })
    }
}

impl IAst for StructNode {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        if analyzer
            .check_identifier_existance(&self.identifier)
            .is_ok()
        {
            analyzer.error(&format!(
                "Identifier \"{}\" defined multiple times",
                &self.identifier
            ));
            return Err(MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR);
        }

        analyzer.scope.push(&self.identifier);
        for internal in self.internals.iter_mut() {
            internal.analyze(analyzer)?;
        }
        analyzer.scope.pop();

        let mut components = Vec::<types::Type>::new();
        for field in self.fields.iter() {
            components.push(field.1.clone());
        }

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::StructDef { components }),
        );

        Ok(())
    }

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

#[derive(Clone, PartialEq)]
pub enum EnumField {
    StructLike(HashMap<Id, types::Type>),
    TupleLike(Vec<types::Type>),
    Common,
}

impl EnumField {
    pub fn parse(parser: &mut Parser) -> Result<Self, &'static str> {
        let next = parser.peek_token();
        match next.lexem {
            TokenType::EndOfLine => Ok(EnumField::Common),

            TokenType::LParen => {
                if let types::Type::Tuple { components } = types::Type::parse_tuple_def(parser)? {
                    Ok(Self::TupleLike(components))
                } else {
                    Err(UNEXPECTED_TOKEN_ERROR_STR)
                }
            }

            TokenType::Lcb => {
                if let Ast::StructDef { node } = StructNode::parse_body_external(parser)? {
                    if !node.internals.is_empty() {
                        parser.error("Internal structs are not allowed here", next.get_location());
                    }

                    return Ok(EnumField::StructLike(node.fields));
                }
                Err(UNEXPECTED_NODE_PARSED_ERROR_STR)
            }

            _ => {
                parser.error(
                    &format!("Unexpected token during parsing enum: {}", next),
                    next.get_location(),
                );
                Err(UNEXPECTED_NODE_PARSED_ERROR_STR)
            }
        }
    }
}

impl IAst for EnumField {
    fn analyze(&mut self, _analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        todo!("EnumField analyzer")
    }

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

#[derive(Clone, PartialEq)]
pub struct EnumNode {
    pub identifier: Id,
    pub fields: HashMap<Id, EnumField>,
    pub internals: Vec<Ast>,
}

impl EnumNode {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, &'static str> {
        let identifier = Self::parse_header(parser)?.identifier;

        if let Ast::EnumDef { mut node } = Self::parse_body_external(parser)? {
            node.identifier = identifier;
            return Ok(Ast::EnumDef { node });
        }

        Err(UNEXPECTED_NODE_PARSED_ERROR_STR)
    }

    pub fn parse_header(parser: &mut Parser) -> Result<Self, &'static str> {
        parser.consume_token(TokenType::KwEnum)?;

        let identifier = parser.consume_identifier()?;

        Ok(EnumNode {
            identifier,
            fields: HashMap::new(),
            internals: Vec::new(),
        })
    }

    pub fn parse_body_external(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(TokenType::Lcb)?;

        let fields = Self::parse_body_internal(parser);

        parser.consume_token(TokenType::Rcb)?;

        fields
    }

    pub fn parse_body_internal(parser: &mut Parser) -> Result<Ast, &'static str> {
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

                    return Err(UNEXPECTED_TOKEN_ERROR_STR);
                }
            }
        }

        Ok(Ast::EnumDef {
            node: Self {
                identifier: String::new(),
                fields,
                internals,
            },
        })
    }
}

impl IAst for EnumNode {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        if let Ok(_ss) = analyzer.check_identifier_existance(&self.identifier) {
            analyzer.error(&format!(
                "Identifier \"{}\" defined multiple times",
                &self.identifier
            ));
            return Err(MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR);
        }

        analyzer.scope.push(&self.identifier);
        for internal in self.internals.iter_mut() {
            internal.analyze(analyzer)?;
        }
        analyzer.scope.pop();

        let mut components = Vec::<EnumField>::new();
        for field in self.fields.iter() {
            components.push(field.1.clone());
        }

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::EnumDef { components }),
        );

        Ok(())
    }

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
