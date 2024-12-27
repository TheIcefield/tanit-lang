use crate::analyzer::SymbolData;
use crate::ast::{
    expressions::{Expression, ExpressionType},
    identifiers::Identifier,
    types::Type,
    Ast, IAst,
};
use crate::codegen::CodeGenStream;
use crate::messages::Message;
use crate::parser::location::Location;
use crate::parser::{token::Lexem, Parser};

use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct VariableNode {
    pub location: Location,
    pub identifier: Identifier,
    pub var_type: Type,
    pub is_global: bool,
    pub is_mutable: bool,
}

impl VariableNode {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, Message> {
        let next = parser.peek_token();
        let location = next.location;

        let is_global = match next.lexem {
            Lexem::KwLet => {
                parser.get_token();
                false
            }

            Lexem::KwStatic => {
                parser.get_token();
                true
            }

            _ => {
                return Err(Message::unexpected_token(
                    next,
                    &[Lexem::KwLet, Lexem::KwStatic],
                ));
            }
        };

        let next = parser.peek_token();
        let is_mutable = match next.lexem {
            Lexem::KwMut => {
                parser.get_token();
                true
            }

            Lexem::KwConst => {
                parser.get_token();
                false
            }

            _ => false,
        };

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        let next = parser.peek_token();

        let mut var_type: Option<Type> = None;
        let mut rvalue: Option<Ast> = None;

        if Lexem::Colon == next.lexem {
            parser.consume_token(Lexem::Colon)?;

            var_type = Some(Type::parse(parser)?);
        }

        let next = parser.peek_token();

        if Lexem::Assign == next.lexem {
            parser.get_token();

            rvalue = Some(Expression::parse(parser)?);
        }

        if var_type.is_none() && rvalue.is_none() {
            return Err(Message::new(
                location,
                &format!(
                    "Variable {} defined without type. Need to specify type or use with rvalue",
                    identifier
                ),
            ));
        }

        if var_type.is_none() && is_global {
            return Err(Message::new(
                location,
                &format!(
                    "Variable {} defined without type, but marked as static. Need to specify type",
                    identifier
                ),
            ));
        }

        if var_type.is_none() && rvalue.is_some() {
            var_type = Some(Type::Auto);
        }

        let var_node = Ast::VariableDef {
            node: Self {
                location,
                identifier,
                var_type: var_type.unwrap_or(Type::Auto),
                is_global,
                is_mutable,
            },
        };

        if let Some(rhs) = rvalue {
            return Ok(Ast::Expression {
                node: Box::new(Expression {
                    location,
                    expr: ExpressionType::Binary {
                        operation: Lexem::Assign,
                        lhs: Box::new(var_node),
                        rhs: Box::new(rhs),
                    },
                }),
            });
        }

        Ok(var_node)
    }

    /* parse function param */
    pub fn parse_param(parser: &mut Parser) -> Result<Self, Message> {
        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        parser.consume_token(Lexem::Colon)?;

        let var_type = Type::parse(parser)?;

        Ok(Self {
            location: identifier.location,
            identifier,
            var_type,
            is_global: false,
            is_mutable: true,
        })
    }
}

impl IAst for VariableNode {
    fn get_type(&self, _analyzer: &mut crate::analyzer::Analyzer) -> Type {
        self.var_type.clone()
    }

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

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::VariableDef {
                var_type: self.var_type.clone(),
                is_mutable: self.is_mutable,
                is_initialization: false,
            }),
        );

        Ok(())
    }

    fn serialize(&self, writer: &mut crate::serializer::XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("variable-definition")?;

        self.identifier.serialize(writer)?;
        writer.put_param("is-global", self.is_global)?;
        writer.put_param("is-mutable", self.is_mutable)?;

        self.var_type.serialize(writer)?;

        writer.end_tag()?;

        Ok(())
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        self.var_type.codegen(stream)?;

        write!(stream, "{}", if self.is_mutable { " " } else { " const " })?;

        self.identifier.codegen(stream)?;

        Ok(())
    }
}

#[test]
fn variables_test() {
    use crate::ast::{functions::FunctionNode, values::ValueType};
    use crate::parser::lexer::Lexer;
    use std::str::FromStr;

    let radian_var_id = Identifier::from_str("radian").unwrap();
    let i32_type_id = Identifier::from_str("i32").unwrap();

    static SRC_PATH: &str = "./examples/values.tt";

    let lexer = Lexer::from_file(SRC_PATH, false).unwrap();

    let mut parser = Parser::new(lexer);

    let res = FunctionNode::parse_def(&mut parser).unwrap();

    let res = if let Ast::FuncDef { node } = &res {
        assert!(node.identifier == Identifier::from_str("main").unwrap());
        assert!(node.parameters.is_empty());

        if let Type::Tuple { components } = &node.return_type {
            assert!(components.is_empty());
        } else {
            panic!("Type expected to be an empty tuple");
        }

        node.body.as_ref()
    } else {
        panic!("res has to be \'FuncDef\'");
    };

    let res = if let Ast::Scope { node } = res.unwrap().as_ref() {
        &node.statements
    } else {
        panic!("res has to be \'LScope\'");
    };

    if let Ast::VariableDef { node } = &res[0] {
        assert!(node.identifier == Identifier::from_str("PI").unwrap());
        assert!(!node.is_mutable);
        assert!(!node.is_global);
        assert_eq!(node.var_type, Type::F32);
    } else {
        panic!("first statement has to be \'variable definition\'");
    }

    if let Ast::Expression { node } = &res[1] {
        let (lhs, rhs) = if let ExpressionType::Binary {
            operation,
            lhs,
            rhs,
        } = &node.as_ref().expr
        {
            assert_eq!(*operation, Lexem::Assign);
            (lhs.as_ref(), rhs.as_ref())
        } else {
            panic!("Expected binary expression");
        };

        if let Ast::VariableDef { node } = lhs {
            assert!(node.identifier == radian_var_id);
            assert!(!node.is_global);
            assert!(!node.is_mutable);
        } else {
            panic!("Expected variable definition")
        }

        if let Ast::Expression { node } = rhs {
            if let ExpressionType::Binary { operation, .. } = &node.as_ref().expr {
                assert_eq!(*operation, Lexem::Slash);
            } else {
                panic!("expected binary expression")
            }
        } else {
            panic!("expected expression")
        }
    } else {
        panic!("second statement has to be \'variable definition\'");
    }

    if let Ast::Expression { node } = &res[2] {
        if let ExpressionType::Binary {
            operation,
            lhs,
            rhs,
        } = &node.as_ref().expr
        {
            assert_eq!(*operation, Lexem::Assign);

            if let Ast::Expression { node } = lhs.as_ref() {
                let (lhs, rhs) = if let ExpressionType::Binary {
                    operation,
                    lhs,
                    rhs,
                } = &node.as_ref().expr
                {
                    assert_eq!(*operation, Lexem::KwAs);
                    (lhs.as_ref(), rhs.as_ref())
                } else {
                    panic!("Binary expression expected")
                };

                if let Ast::VariableDef { node } = lhs {
                    assert!(node.identifier == Identifier::from_str("ceil").unwrap());
                    assert!(!node.is_global);
                    assert!(!node.is_mutable);
                } else {
                    panic!("Expected variable definition")
                }

                if let Ast::Value { node } = rhs {
                    if let ValueType::Identifier(id) = &node.value {
                        assert!(*id == i32_type_id);
                    } else {
                        panic!("Expected identifier")
                    }
                } else {
                    panic!("Expected value")
                }
            }

            let expr = if let Ast::Expression { node } = rhs.as_ref() {
                node.as_ref()
            } else {
                panic!("rhs expected to be \'Expression\'");
            };

            if let ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } = &expr.expr
            {
                assert_eq!(*operation, Lexem::KwAs);

                if let Ast::Value { node } = lhs.as_ref() {
                    if let ValueType::Identifier(id) = &node.value {
                        assert!(*id == Identifier::from_str("radian").unwrap())
                    }
                } else {
                    panic!("rhs has to be \'Expression\'");
                };

                assert!(matches!(rhs.as_ref(), Ast::TypeDecl { node: Type::I32 }));
            } else {
                panic!("Expected binary expression");
            }
        }
    } else {
        panic!("third statement has to be \'expression\'");
    }
}
