use crate::analyzer::SymbolData;
use crate::ast::{identifiers::Identifier, types::Type, Ast, IAst, Stream};
use crate::codegen::{CodeGenMode, CodeGenStream};
use crate::error_listener::{
    MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR, UNEXPECTED_NODE_PARSED_ERROR_STR,
    UNEXPECTED_TOKEN_ERROR_STR,
};
use crate::lexer::Lexem;
use crate::parser::{put_intent, Parser};

use std::collections::HashMap;
use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct StructNode {
    pub identifier: Identifier,
    pub fields: HashMap<Identifier, Type>,
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
        parser.consume_token(Lexem::KwStruct)?;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        Ok(StructNode {
            identifier,
            fields: HashMap::new(),
            internals: Vec::new(),
        })
    }

    pub fn parse_body_external(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(Lexem::Lcb)?;

        let fields = Self::parse_body_internal(parser);

        parser.consume_token(Lexem::Rcb)?;

        fields
    }

    pub fn parse_body_internal(parser: &mut Parser) -> Result<Ast, &'static str> {
        let mut fields = HashMap::<Identifier, Type>::new();
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
                    internals.push(StructNode::parse_def(parser)?);
                }

                Lexem::KwEnum => {
                    internals.push(EnumNode::parse_def(parser)?);
                }

                Lexem::Identifier(id) => {
                    let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

                    if fields.contains_key(&identifier) {
                        parser.error(
                            &format!("Struct has already field with identifier {}", id),
                            next.get_location(),
                        );
                        continue;
                    }

                    parser.consume_token(Lexem::Colon)?;

                    fields.insert(identifier, Type::parse(parser)?);
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
                identifier: Identifier::new(),
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

        analyzer.scope.push(&format!("@s.{}", &self.identifier));
        for internal in self.internals.iter_mut() {
            internal.analyze(analyzer)?;
        }

        let mut components = Vec::<Type>::new();
        for field in self.fields.iter() {
            components.push(field.1.clone());
        }

        analyzer.scope.pop();

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
            writeln!(stream, "{}<field {}>", put_intent(intent + 1), field.0)?;

            field.1.traverse(stream, intent + 2)?;

            writeln!(stream, "{}</field>", put_intent(intent + 1))?;
        }

        writeln!(stream, "{}</struct-def>", put_intent(intent))?;

        Ok(())
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::HeaderOnly;

        writeln!(stream, "typedef struct {{")?;
        for (field_id, field_type) in self.fields.iter() {
            field_type.codegen(stream)?;
            write!(stream, " ")?;
            field_id.codegen(stream)?;
            writeln!(stream, ";")?;
        }
        write!(stream, "}} ")?;

        self.identifier.codegen(stream)?;

        writeln!(stream, ";")?;

        stream.mode = old_mode;
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub enum EnumField {
    StructLike(HashMap<Identifier, Type>),
    TupleLike(Vec<Type>),
    Common,
}

impl EnumField {
    pub fn parse(parser: &mut Parser) -> Result<Self, &'static str> {
        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);
        let res = Self::parse_internal(parser);
        parser.set_ignore_nl_option(old_opt);

        res
    }

    fn parse_internal(parser: &mut Parser) -> Result<Self, &'static str> {
        let next = parser.peek_token();
        match next.lexem {
            Lexem::EndOfLine => Ok(EnumField::Common),

            Lexem::LParen => {
                if let Type::Tuple { components } = Type::parse_tuple_def(parser)? {
                    Ok(Self::TupleLike(components))
                } else {
                    Err(UNEXPECTED_TOKEN_ERROR_STR)
                }
            }

            Lexem::Lcb => {
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

    fn codegen(&self, _stream: &mut CodeGenStream) -> std::io::Result<()> {
        unimplemented!()
    }
}

#[derive(Clone, PartialEq)]
pub struct EnumNode {
    pub identifier: Identifier,
    pub fields: HashMap<Identifier, EnumField>,
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
        parser.consume_token(Lexem::KwEnum)?;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        Ok(EnumNode {
            identifier,
            fields: HashMap::new(),
            internals: Vec::new(),
        })
    }

    pub fn parse_body_external(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(Lexem::Lcb)?;
        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);
        let fields = Self::parse_body_internal(parser);
        parser.set_ignore_nl_option(old_opt);

        parser.consume_token(Lexem::Rcb)?;

        fields
    }

    pub fn parse_body_internal(parser: &mut Parser) -> Result<Ast, &'static str> {
        let mut fields = HashMap::<Identifier, EnumField>::new();
        let mut internals = Vec::<Ast>::new();

        loop {
            let next = parser.peek_token();

            match &next.lexem {
                Lexem::Rcb => break,

                Lexem::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                Lexem::KwStruct => internals.push(StructNode::parse_def(parser)?),

                Lexem::KwEnum => internals.push(EnumNode::parse_def(parser)?),

                Lexem::Identifier(id) => {
                    let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

                    if fields.contains_key(&identifier) {
                        parser.error(
                            &format!("Enum has already field with identifier \"{}\"", id),
                            next.get_location(),
                        );
                        continue;
                    }

                    fields.insert(identifier, EnumField::parse(parser)?);

                    parser.consume_new_line()?;
                }

                Lexem::Lcb => {
                    parser.error(
                        &format!(
                            "{}\nHelp: {}{}",
                            "Unexpected token: \"{\" during parsing enum fields.",
                            "If you tried to declare struct-like field, place \"{\" ",
                            "in the same line with name of the field."
                        ),
                        next.get_location(),
                    );

                    return Err(UNEXPECTED_TOKEN_ERROR_STR);
                }

                _ => {
                    parser.error(
                        &format!(
                            "Unexpected token: \"{}\" during parsing enum fields",
                            next.lexem
                        ),
                        next.get_location(),
                    );

                    return Err(UNEXPECTED_TOKEN_ERROR_STR);
                }
            }
        }

        Ok(Ast::EnumDef {
            node: Self {
                identifier: Identifier::new(),
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

        analyzer.scope.push(&format!("@e.{}", &self.identifier));
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

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::HeaderOnly;

        writeln!(stream, "typedef enum {{")?;
        for (field_id, _) in self.fields.iter() {
            field_id.codegen(stream)?;
            writeln!(stream, ",")?;
        }
        write!(stream, "}} ")?;

        self.identifier.codegen(stream)?;

        writeln!(stream, ";")?;

        stream.mode = old_mode;
        Ok(())
    }
}
