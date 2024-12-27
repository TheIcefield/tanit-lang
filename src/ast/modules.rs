use crate::analyzer::SymbolData;
use crate::ast::{
    identifiers::{Identifier, IdentifierType},
    scopes::Scope,
    Ast, IAst,
};
use crate::codegen::CodeGenStream;
use crate::messages::Message;
use crate::parser::location::Location;
use crate::parser::{lexer::Lexer, token::Lexem, Parser};

#[derive(Clone, PartialEq)]
pub struct ModuleNode {
    pub location: Location,
    pub identifier: Identifier,
    pub body: Box<Ast>,
}

impl ModuleNode {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::parse_header(parser)?;

        node.body = Box::new(Scope::parse_global(parser)?);

        Ok(Ast::ModuleDef { node })
    }

    pub fn parse_header(parser: &mut Parser) -> Result<Self, Message> {
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

    pub fn parse_ext_body(
        identifier: &Identifier,
        parser: &mut Parser,
    ) -> Result<Box<Ast>, Message> {
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

impl IAst for ModuleNode {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        let identifier = match &self.identifier.identifier {
            IdentifierType::Common(id) => id.clone(),
            IdentifierType::Complex(..) => {
                return Err(Message::new(
                    self.location,
                    &format!(
                        "Expected common identifier, actually complex: {}",
                        self.identifier
                    ),
                ));
            }
        };

        if analyzer
            .check_identifier_existance(&self.identifier)
            .is_ok()
        {
            return Err(Message::new(
                self.location,
                &format!("Identifier \"{}\" defined multiple times", &self.identifier),
            ));
        }

        analyzer.scope.push(&identifier);
        self.body.analyze(analyzer)?;
        analyzer.scope.pop();

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::ModuleDef {
                full_name: vec![identifier],
            }),
        );

        Ok(())
    }

    fn serialize(&self, writer: &mut crate::serializer::XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("module-definition")?;

        self.body.serialize(writer)?;

        writer.end_tag()?;

        Ok(())
    }

    fn codegen(&self, _stream: &mut CodeGenStream) -> std::io::Result<()> {
        unimplemented!()
    }
}

#[test]
fn module_test() {
    use std::str::FromStr;

    static SRC_PATH: &str = "./examples/modules.tt";

    let lexer = Lexer::from_file(SRC_PATH, false).unwrap();

    let mut parser = Parser::new(lexer);

    let res = ModuleNode::parse_def(&mut parser).unwrap();

    let res = if let Ast::ModuleDef { node } = &res {
        assert!(node.identifier == Identifier::from_str("M1").unwrap());
        &node.body
    } else {
        panic!("res should be \'ModuleDef\'");
    };

    let res = if let Ast::Scope { node } = res.as_ref() {
        &node.statements[0]
    } else {
        panic!("res should be \'global scope\'");
    };

    if let Ast::ModuleDef { node } = res {
        assert!(node.identifier == Identifier::from_str("M2").unwrap());
    } else {
        panic!("res should be \'ModuleDef\'");
    };
}
