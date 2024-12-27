use crate::analyzer::SymbolData;
use crate::ast::{identifiers::Identifier, types::Type, Ast, IAst};
use crate::codegen::{CodeGenMode, CodeGenStream};
use crate::messages::Message;
use crate::parser::location::Location;
use crate::parser::token::Lexem;
use crate::parser::Parser;

use std::collections::HashMap;
use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct StructNode {
    pub location: Location,
    pub identifier: Identifier,
    pub fields: HashMap<Identifier, Type>,
    pub internals: Vec<Ast>,
}

impl StructNode {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, Message> {
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

        Ok(StructNode {
            location,
            identifier,
            fields: HashMap::new(),
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

impl IAst for StructNode {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        if analyzer
            .check_identifier_existance(&self.identifier)
            .is_ok()
        {
            return Err(Message::multiple_ids(
                self.location,
                &self.identifier.get_string(),
            ));
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

    fn serialize(&self, writer: &mut crate::serializer::XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("struct-definition")?;

        self.identifier.serialize(writer)?;

        for internal in self.internals.iter() {
            internal.serialize(writer)?;
        }

        for (field_id, field_type) in self.fields.iter() {
            writer.begin_tag("field")?;

            field_id.serialize(writer)?;
            field_type.serialize(writer)?;

            writer.end_tag()?;
        }

        writer.end_tag()?;

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
            Lexem::EndOfLine => Ok(EnumField::Common),

            Lexem::LParen => {
                if let Type::Tuple { components } = Type::parse_tuple_def(parser)? {
                    Ok(Self::TupleLike(components))
                } else {
                    Err(Message::unexpected_token(next, &[]))
                }
            }

            Lexem::Lcb => {
                if let Ast::StructDef { node } = StructNode::parse_body_external(parser)? {
                    if !node.internals.is_empty() {
                        parser.error(Message::new(
                            next.location,
                            "Internal structs are not allowed here",
                        ));
                    }

                    return Ok(EnumField::StructLike(node.fields));
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

impl IAst for EnumField {
    fn analyze(&mut self, _analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        todo!("EnumField analyzer")
    }

    fn serialize(&self, writer: &mut crate::serializer::XmlWriter) -> std::io::Result<()> {
        match self {
            Self::StructLike(s) => {
                for (field_id, field_type) in s.iter() {
                    writer.begin_tag("field")?;

                    field_id.serialize(writer)?;
                    field_type.serialize(writer)?;

                    writer.end_tag()?;
                }
            }
            Self::TupleLike(tuple_field) => {
                for tuple_component in tuple_field.iter() {
                    tuple_component.serialize(writer)?;
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
    pub location: Location,
    pub identifier: Identifier,
    pub fields: HashMap<Identifier, EnumField>,
    pub internals: Vec<Ast>,
}

impl EnumNode {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, Message> {
        let identifier = Self::parse_header(parser)?.identifier;

        if let Ast::EnumDef { mut node } = Self::parse_body_external(parser)? {
            node.identifier = identifier;
            return Ok(Ast::EnumDef { node });
        }

        unreachable!()
    }

    pub fn parse_header(parser: &mut Parser) -> Result<Self, Message> {
        let location = parser.consume_token(Lexem::KwEnum)?.location;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        Ok(EnumNode {
            location,
            identifier,
            fields: HashMap::new(),
            internals: Vec::new(),
        })
    }

    pub fn parse_body_external(parser: &mut Parser) -> Result<Ast, Message> {
        parser.consume_token(Lexem::Lcb)?;
        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);
        let fields = Self::parse_body_internal(parser);
        parser.set_ignore_nl_option(old_opt);

        parser.consume_token(Lexem::Rcb)?;

        fields
    }

    pub fn parse_body_internal(parser: &mut Parser) -> Result<Ast, Message> {
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
                        parser.error(Message::new(
                            next.location,
                            &format!("Enum has already field with identifier \"{}\"", id),
                        ));
                        continue;
                    }

                    fields.insert(identifier, EnumField::parse(parser)?);

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

        Ok(Ast::EnumDef {
            node: Self {
                location: Location::new(),
                identifier: Identifier::new(),
                fields,
                internals,
            },
        })
    }
}

impl IAst for EnumNode {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        if let Ok(_ss) = analyzer.check_identifier_existance(&self.identifier) {
            return Err(Message::multiple_ids(
                self.location,
                &self.identifier.get_string(),
            ));
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

    fn serialize(&self, writer: &mut crate::serializer::XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("enum-definition")?;

        self.identifier.serialize(writer)?;

        for internal in self.internals.iter() {
            internal.serialize(writer)?;
        }

        for (field_id, field) in self.fields.iter() {
            writer.begin_tag("field")?;

            field_id.serialize(writer)?;

            if EnumField::Common == *field {
                writer.end_tag()?;
                continue;
            }

            field.serialize(writer)?;

            writer.end_tag()?;
        }

        writer.end_tag()?;

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

#[test]
fn struct_test() {
    use crate::parser::lexer::Lexer;
    use std::str::FromStr;

    static SRC_PATH: &str = "./examples/structs.tt";

    let lexer = Lexer::from_file(SRC_PATH, false).unwrap();

    let mut parser = Parser::new(lexer);

    let res = StructNode::parse_def(&mut parser).unwrap();

    if let Ast::StructDef { node } = res {
        assert!(node.identifier == Identifier::from_str("S1").unwrap());

        let field_type = node
            .fields
            .get(&Identifier::from_str("f1").unwrap())
            .unwrap();
        assert!(matches!(field_type, Type::I32));

        let field_type = node
            .fields
            .get(&Identifier::from_str("f2").unwrap())
            .unwrap();

        if let Type::Template {
            identifier,
            arguments,
        } = &field_type
        {
            let expected_id = Identifier::from_str("Vec").unwrap();

            assert!(*identifier == expected_id);
            assert_eq!(arguments.len(), 1);
            assert_eq!(arguments[0], Type::I32);
        } else {
            panic!("wrong type");
        }
    } else {
        panic!("res should be \'StructDef\'");
    };
}

#[test]
fn enum_test() {
    use crate::parser::lexer::Lexer;
    use std::str::FromStr;

    static SRC_PATH: &str = "./examples/structs.tt";

    let lexer = Lexer::from_file(SRC_PATH, false).unwrap();

    let mut parser = Parser::new(lexer);

    StructNode::parse_def(&mut parser).unwrap();

    let res = EnumNode::parse_def(&mut parser).unwrap();

    if let Ast::EnumDef { node } = &res {
        assert!(node.identifier == Identifier::from_str("E1").unwrap());

        assert!(matches!(
            node.fields.get(&Identifier::from_str("f1").unwrap()),
            Some(&EnumField::Common)
        ));

        if let EnumField::TupleLike(components) = node
            .fields
            .get(&Identifier::from_str("f2").unwrap())
            .unwrap()
        {
            assert_eq!(components.len(), 2);
            assert_eq!(components[0], Type::I32);
            assert_eq!(components[1], Type::I32);
        } else {
            panic!("wrong type");
        }

        let field = node
            .fields
            .get(&Identifier::from_str("f3").unwrap())
            .unwrap();
        if let EnumField::StructLike(components) = &field {
            assert_eq!(components.len(), 2);
            assert!(matches!(
                components.get(&Identifier::from_str("f1").unwrap()),
                Some(&Type::I32)
            ));
            assert!(matches!(
                components.get(&Identifier::from_str("f2").unwrap()),
                Some(&Type::F32)
            ));
        } else {
            panic!("wrong type");
        }
    } else {
        panic!("res should be \'EnumDef\'");
    };
}
