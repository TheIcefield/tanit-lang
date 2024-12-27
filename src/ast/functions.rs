use crate::analyzer::SymbolData;
use crate::ast::{identifiers::Identifier, scopes, types, variables::VariableNode, Ast, IAst};
use crate::codegen::{CodeGenMode, CodeGenStream};
use crate::messages::Message;
use crate::parser::{location::Location, token::Lexem, Parser};

use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct FunctionNode {
    pub location: Location,
    pub identifier: Identifier,
    pub return_type: types::Type,
    pub parameters: Vec<Ast>,
    pub body: Option<Box<Ast>>,
}

impl FunctionNode {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::parse_header(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            Lexem::Lcb => {
                node.body = Some(Box::new(scopes::Scope::parse_local(parser)?));
            }

            _ => {
                return Err(Message::unexpected_token(
                    next,
                    &[Lexem::Lcb, Lexem::EndOfLine],
                ));
            }
        }

        Ok(Ast::FuncDef { node })
    }

    pub fn parse_header(parser: &mut Parser) -> Result<Self, Message> {
        let location = parser.consume_token(Lexem::KwFunc)?.location;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        let parameters = Self::parse_header_params(parser)?;

        let next = parser.peek_token();
        let return_type = if next.lexem == Lexem::Arrow {
            parser.get_token();
            types::Type::parse(parser)?
        } else {
            types::Type::Tuple {
                components: Vec::<types::Type>::new(),
            }
        };

        Ok(Self {
            location,
            identifier,
            return_type,
            parameters,
            body: None,
        })
    }

    pub fn parse_header_params(parser: &mut Parser) -> Result<Vec<Ast>, Message> {
        parser.consume_token(Lexem::LParen)?;

        let mut parameters = Vec::<Ast>::new();
        loop {
            let next = parser.peek_token();

            if next.is_identifier() {
                parameters.push(Ast::VariableDef {
                    node: VariableNode::parse_param(parser)?,
                });

                let next = parser.peek_token();
                if next.lexem == Lexem::Comma {
                    parser.get_token();
                } else if next.lexem == Lexem::RParen {
                    continue;
                } else {
                    return Err(Message::unexpected_token(next, &[]));
                }
            } else if next.lexem == Lexem::RParen {
                parser.get_token();
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        Ok(parameters)
    }
}

impl IAst for FunctionNode {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        if let Ok(_ss) = analyzer.check_identifier_existance(&self.identifier) {
            return Err(Message::multiple_ids(
                self.location,
                &self.identifier.get_string(),
            ));
        }

        analyzer.scope.push(&format!("@f.{}", &self.identifier));

        let mut arguments = Vec::<(String, types::Type)>::new();
        for p in self.parameters.iter_mut() {
            if let Ast::VariableDef { node } = p {
                arguments.push((node.identifier.get_string(), node.var_type.clone()));
                p.analyze(analyzer)?;
            }
        }

        analyzer.scope.pop();

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::FunctionDef {
                args: arguments.clone(),
                return_type: self.return_type.clone(),
                is_declaration: self.body.is_some(),
            }),
        );

        analyzer.scope.push(&format!("@f.{}", &self.identifier));

        if let Some(body) = &mut self.body {
            if let Ast::Scope { node } = body.as_mut() {
                for stmt in node.statements.iter_mut() {
                    stmt.analyze(analyzer)?;
                }
            }
        }

        analyzer.scope.pop();

        Ok(())
    }

    fn serialize(&self, writer: &mut crate::serializer::XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("function-definition")?;

        self.identifier.serialize(writer)?;

        writer.begin_tag("return-type")?;
        self.return_type.serialize(writer)?;
        writer.end_tag()?;

        if !self.parameters.is_empty() {
            writer.begin_tag("parameters")?;
            for param in self.parameters.iter() {
                param.serialize(writer)?;
            }
            writer.end_tag()?;
        }

        if let Some(body) = &self.body {
            body.serialize(writer)?;
        }

        writer.end_tag()?;

        Ok(())
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = if self.body.is_some() {
            CodeGenMode::Both
        } else {
            CodeGenMode::HeaderOnly
        };

        self.return_type.codegen(stream)?;

        write!(stream, " ")?;

        self.identifier.codegen(stream)?;

        // generate parameters
        write!(stream, "(")?;
        if !self.parameters.is_empty() {
            self.parameters[0].codegen(stream)?;
        }

        for param in self.parameters.iter().skip(1) {
            write!(stream, ", ")?;
            param.codegen(stream)?;
        }
        write!(stream, ")")?;

        stream.mode = CodeGenMode::HeaderOnly;
        writeln!(stream, ";")?;

        if let Some(body) = &self.body {
            stream.mode = CodeGenMode::SourceOnly;
            body.codegen(stream)?;
        }

        stream.mode = old_mode;
        Ok(())
    }
}

#[test]
fn functions_test() {
    use crate::ast::expressions::ExpressionType;
    use crate::parser::lexer::Lexer;

    static SRC_PATH: &str = "./examples/functions.tt";

    let lexer = Lexer::from_file(SRC_PATH, true).unwrap();

    let mut parser = Parser::new(lexer);

    {
        let func = FunctionNode::parse_def(&mut parser).unwrap();

        let scope = if let Ast::FuncDef { node } = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let Ast::Scope { node } = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        assert!(matches!(node, Ast::ReturnStmt { .. }));
    }

    {
        let func = FunctionNode::parse_def(&mut parser).unwrap();

        let scope = if let Ast::FuncDef { node } = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let Ast::Scope { node } = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        let (lhs, rhs) = if let Ast::Expression { node } = node {
            if let ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } = &node.as_ref().expr
            {
                assert_eq!(*operation, Lexem::Assign);
                (lhs.as_ref(), rhs.as_ref())
            } else {
                panic!("Expression expected to be binary");
            }
        } else {
            panic!("Expected expression");
        };

        assert!(matches!(lhs, Ast::VariableDef { .. }));
        assert!(matches!(rhs, Ast::Value { .. }));
    }
}
