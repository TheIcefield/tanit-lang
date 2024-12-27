use crate::ast::{expressions::Expression, scopes::Scope, Ast, IAst};
use crate::codegen::{CodeGenMode, CodeGenStream};
use crate::messages::Message;
use crate::parser::{location::Location, token::Lexem, Parser};
use crate::serializer;

use std::io::Write;

#[derive(Clone, PartialEq)]
pub enum BranchType {
    Loop {
        body: Box<Ast>,
        condition: Option<Box<Ast>>,
    },
    IfElse {
        condition: Box<Ast>,
        main_body: Box<Ast>,
        else_body: Option<Box<Ast>>,
    },
}

#[derive(Clone, PartialEq)]
pub struct Branch {
    location: Location,
    branch: BranchType,
}

impl Branch {
    pub fn parse_loop(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwLoop)?.location;

        let body = Box::new(Scope::parse_local(parser)?);

        Ok(Ast::BranchStmt {
            node: Self {
                location,
                branch: BranchType::Loop {
                    body,
                    condition: None,
                },
            },
        })
    }

    pub fn parse_while(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwWhile)?.location;

        let condition = Some(Box::new(Expression::parse(parser)?));

        let body = Box::new(Scope::parse_local(parser)?);

        Ok(Ast::BranchStmt {
            node: Self {
                location,
                branch: BranchType::Loop { body, condition },
            },
        })
    }

    pub fn parse_if(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwIf)?.location;

        let condition = Box::new(Expression::parse(parser)?);

        let main_body = Box::new(Scope::parse_local(parser)?);

        let else_body = if parser.peek_token().lexem == Lexem::KwElse {
            parser.get_token();

            let next = parser.peek_token();
            match next.lexem {
                Lexem::KwIf => Some(Box::new(Self::parse_if(parser)?)),
                Lexem::Lcb => Some(Box::new(Scope::parse_local(parser)?)),
                _ => {
                    parser.error(Message::unexpected_token(next, &[Lexem::KwIf, Lexem::Lcb]));
                    None
                }
            }
        } else {
            None
        };

        Ok(Ast::BranchStmt {
            node: Self {
                location,
                branch: BranchType::IfElse {
                    condition,
                    main_body,
                    else_body,
                },
            },
        })
    }
}

impl IAst for Branch {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        match &mut self.branch {
            BranchType::IfElse {
                condition,
                main_body,
                else_body,
            } => {
                condition.analyze(analyzer)?;
                main_body.analyze(analyzer)?;
                if let Some(else_body) = else_body.as_mut() {
                    else_body.analyze(analyzer)?;
                }

                Ok(())
            }
            BranchType::Loop { body, condition } => {
                let cnt = analyzer.counter();
                analyzer.scope.push(&format!("@l.{}", cnt));

                if let Some(cond) = condition {
                    cond.analyze(analyzer)?;
                }

                if let Ast::Scope { node } = body.as_mut() {
                    for stmt in node.statements.iter_mut() {
                        stmt.analyze(analyzer)?;
                    }
                }

                analyzer.scope.pop();

                Ok(())
            }
        }
    }

    fn serialize(&self, writer: &mut crate::serializer::XmlWriter) -> std::io::Result<()> {
        match &self.branch {
            BranchType::Loop { body, condition } => {
                writer.begin_tag("loop")?;

                if let Some(cond) = condition {
                    writer.begin_tag("condition")?;

                    cond.serialize(writer)?;

                    writer.end_tag()?;
                }

                body.serialize(writer)?;

                writer.end_tag()?;
            }
            BranchType::IfElse {
                condition,
                main_body,
                else_body,
            } => {
                writer.begin_tag("branch")?;

                writer.begin_tag("condition")?;
                condition.serialize(writer)?;
                writer.end_tag()?;

                writer.begin_tag("true")?;
                main_body.serialize(writer)?;
                writer.end_tag()?;

                if let Some(else_body) = else_body {
                    writer.begin_tag("false")?;
                    else_body.serialize(writer)?;
                    writer.end_tag()?;
                }

                writer.end_tag()?;
            }
        }

        Ok(())
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;
        match &self.branch {
            BranchType::IfElse {
                condition,
                main_body,
                else_body,
            } => {
                write!(stream, "if (")?;
                condition.codegen(stream)?;
                writeln!(stream, ")")?;

                main_body.codegen(stream)?;

                if let Some(else_body) = else_body {
                    writeln!(stream, "else")?;
                    else_body.codegen(stream)?;
                }
            }
            BranchType::Loop { body, condition } => {
                write!(stream, "while (")?;

                if let Some(condition) = condition {
                    condition.codegen(stream)?;
                } else {
                    write!(stream, "1")?;
                }
                writeln!(stream, ")")?;

                body.codegen(stream)?;
            }
        }
        stream.mode = old_mode;
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub struct Break {
    pub location: Location,
    pub expr: Option<Box<Ast>>,
}

impl Break {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwBreak)?.location;

        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);

        let mut node = Break {
            location,
            expr: None,
        };

        match parser.peek_token().lexem {
            Lexem::EndOfLine => {}
            _ => node.expr = Some(Box::new(Expression::parse(parser)?)),
        }

        parser.set_ignore_nl_option(old_opt);

        Ok(Ast::BreakStmt { node })
    }
}

impl IAst for Break {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        let mut in_loop = false;
        for s in analyzer.scope.iter().rev() {
            if s.starts_with("@l.") {
                in_loop = true;
                break;
            }
        }

        if let Some(expr) = &mut self.expr {
            expr.analyze(analyzer)?
        }

        if !in_loop {
            return Err(Message::new(self.location, "Unexpected break statement"));
        }

        Ok(())
    }

    fn serialize(&self, writer: &mut serializer::XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("break-statement")?;

        if let Some(expr) = &self.expr {
            expr.serialize(writer)?;
        }

        writer.end_tag()?;

        Ok(())
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;

        write!(stream, "break")?;

        stream.mode = old_mode;
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub struct Continue {
    location: Location,
}

impl Continue {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwContinue)?.location;

        Ok(Ast::ContinueStmt {
            node: Self { location },
        })
    }
}

impl IAst for Continue {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        let mut in_loop = false;
        for s in analyzer.scope.iter().rev() {
            if s.starts_with("@l.") {
                in_loop = true;
                break;
            }
        }

        if !in_loop {
            return Err(Message::new(self.location, "Unexpected continue statement"));
        }

        Ok(())
    }

    fn serialize(&self, writer: &mut serializer::XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("continue-statement")?;
        writer.end_tag()
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;

        write!(stream, "continue")?;

        stream.mode = old_mode;
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub struct Return {
    pub location: Location,
    pub expr: Option<Box<Ast>>,
}

impl Return {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwReturn)?.location;

        let mut node = Return {
            location,
            expr: None,
        };

        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);

        match parser.peek_token().lexem {
            Lexem::EndOfLine => {}
            _ => node.expr = Some(Box::new(Expression::parse(parser)?)),
        }

        parser.set_ignore_nl_option(old_opt);

        Ok(Ast::ReturnStmt { node })
    }
}

impl IAst for Return {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        let mut in_func = false;
        for s in analyzer.scope.iter().rev() {
            if s.starts_with("@f.") {
                in_func = true;
                break;
            }
        }

        if let Some(expr) = &mut self.expr {
            expr.analyze(analyzer)?;
        }

        if !in_func {
            return Err(Message::new(self.location, "Unexpected return statement"));
        }

        Ok(())
    }

    fn serialize(&self, writer: &mut serializer::XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("return-statement")?;

        if let Some(expr) = &self.expr {
            expr.serialize(writer)?;
        }

        writer.end_tag()?;

        Ok(())
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;

        write!(stream, "return ")?;
        if let Some(expr) = self.expr.as_ref() {
            expr.codegen(stream)?;
        }

        stream.mode = old_mode;
        Ok(())
    }
}
