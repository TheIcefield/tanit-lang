use super::{Branch, BranchType, Break, Continue, Return};
use crate::analyzer::{Analyze, Analyzer};
use crate::ast::Ast;
use crate::messages::Message;

impl Analyze for Branch {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
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
}

impl Analyze for Break {
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
}

impl Analyze for Continue {
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
}

impl Analyze for Return {
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
}
