use crate::ast::{expressions::Expression, scopes, Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Parser};

use std::io::Write;

#[derive(Clone, PartialEq)]
pub enum Branch {
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

impl Branch {
    pub fn parse_loop(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(TokenType::KwLoop)?;

        let body = Box::new(scopes::Scope::parse_local(parser)?);

        Ok(Ast::BranchStmt {
            node: Self::Loop {
                body,
                condition: None,
            },
        })
    }

    pub fn parse_while(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(TokenType::KwWhile)?;

        let condition = Expression::parse(parser)?;

        let body = Box::new(scopes::Scope::parse_local(parser)?);

        Ok(Ast::BranchStmt {
            node: Self::Loop {
                body,
                condition: Some(Box::new(condition)),
            },
        })
    }

    pub fn parse_if(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(TokenType::KwIf)?;

        let condition = Box::new(Expression::parse(parser)?);

        let main_body = Box::new(scopes::Scope::parse_local(parser)?);

        let else_body = if parser.peek_token().lexem == TokenType::KwElse {
            parser.get_token();

            let next = parser.peek_token();
            match next.lexem {
                TokenType::KwIf => Some(Box::new(Self::parse_if(parser)?)),
                TokenType::Lcb => Some(Box::new(scopes::Scope::parse_local(parser)?)),
                _ => {
                    parser.error(
                        &format!("Unexpected token \"{}\" in branch expression", next),
                        next.get_location(),
                    );
                    None
                }
            }
        } else {
            None
        };

        Ok(Ast::BranchStmt {
            node: Self::IfElse {
                condition,
                main_body,
                else_body,
            },
        })
    }
}

impl IAst for Branch {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        match self {
            Self::IfElse {
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
            Self::Loop { body, condition } => {
                if let Some(cond) = condition {
                    cond.analyze(analyzer)?;
                }
                body.analyze(analyzer)
            }
        }
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match self {
            Self::Loop { body, condition } => {
                writeln!(stream, "{}<loop>", put_intent(intent))?;

                if let Some(cond) = condition {
                    writeln!(stream, "{}<condition>", put_intent(intent + 1))?;

                    cond.traverse(stream, intent + 2)?;

                    writeln!(stream, "{}</condition>", put_intent(intent + 1))?;
                }

                body.traverse(stream, intent + 1)?;

                writeln!(stream, "{}</loop>", put_intent(intent))?;
            }
            Self::IfElse {
                condition,
                main_body,
                else_body,
            } => {
                writeln!(stream, "{}<branch>", put_intent(intent))?;

                {
                    writeln!(stream, "{}<condition>", put_intent(intent + 1))?;

                    condition.traverse(stream, intent + 2)?;

                    writeln!(stream, "{}</condition>", put_intent(intent + 1))?;
                }

                {
                    writeln!(stream, "{}<true>", put_intent(intent + 1))?;

                    main_body.traverse(stream, intent + 2)?;

                    writeln!(stream, "{}</true>", put_intent(intent + 1))?;
                }

                if let Some(else_body) = else_body {
                    writeln!(stream, "{}<false>", put_intent(intent + 1))?;

                    else_body.traverse(stream, intent + 2)?;

                    writeln!(stream, "{}</false>", put_intent(intent + 1))?;
                }

                writeln!(stream, "{}</branch>", put_intent(intent))?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub struct Break {
    pub expr: Option<Box<Ast>>,
}

impl Break {
    pub fn parse(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(TokenType::KwBreak)?;

        let mut node = Break { expr: None };

        match parser.peek_token().lexem {
            TokenType::EndOfLine => {}
            _ => node.expr = Some(Box::new(Expression::parse(parser)?)),
        }

        Ok(Ast::BreakStmt { node })
    }
}

impl IAst for Break {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        if let Some(expr) = &mut self.expr {
            expr.analyze(analyzer)?
        }

        Ok(())
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match &self.expr {
            Some(cond) => {
                writeln!(stream, "{}<break>", put_intent(intent))?;
                cond.traverse(stream, intent + 1)?;
                writeln!(stream, "{}</break>", put_intent(intent))?;
            }
            _ => {
                writeln!(stream, "{}<break/>", put_intent(intent))?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub struct Continue {}

impl Continue {
    pub fn parse(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(TokenType::KwContinue)?;

        Ok(Ast::ContinueStmt { node: Self {} })
    }
}

impl IAst for Continue {
    fn analyze(&mut self, _analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        Ok(())
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(stream, "{}<continue/>", put_intent(intent))
    }
}

#[derive(Clone, PartialEq)]
pub struct Return {
    pub expr: Option<Box<Ast>>,
}

impl Return {
    pub fn parse(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(TokenType::KwReturn)?;

        let mut node = Return { expr: None };

        match parser.peek_token().lexem {
            TokenType::EndOfLine => {}
            _ => node.expr = Some(Box::new(Expression::parse(parser)?)),
        }

        Ok(Ast::ReturnStmt { node })
    }
}

impl IAst for Return {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        if let Some(expr) = &mut self.expr {
            expr.analyze(analyzer)?;
        }

        Ok(())
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match &self.expr {
            Some(cond) => {
                writeln!(stream, "{}<return>", put_intent(intent))?;
                cond.traverse(stream, intent + 1)?;
                writeln!(stream, "{}</return>", put_intent(intent))?;
            }
            _ => {
                writeln!(stream, "{}<return/>", put_intent(intent))?;
            }
        }

        Ok(())
    }
}
