use super::{Branch, BranchType, Interupter, InterupterType};
use crate::ast::Ast;

use tanitc_analyzer::{Analyze, Analyzer};
use tanitc_messages::Message;

impl Branch {
    fn analyze_body(body: &mut Ast, analyzer: &mut Analyzer) -> Result<(), Message> {
        if let Ast::Scope(node) = body {
            for stmt in node.statements.iter_mut() {
                stmt.analyze(analyzer)?;
            }
        }

        Ok(())
    }

    fn analyze_condition(condition: &mut Ast, analyzer: &mut Analyzer) -> Result<(), Message> {
        if let Ast::Expression(node) = condition {
            node.analyze(analyzer)?;
        }

        Ok(())
    }
}

impl Analyze for Branch {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        match &mut self.branch {
            BranchType::While { body, condition } => {
                let cnt = analyzer.counter();
                analyzer.scope.push(format!("@l.{}", cnt));

                condition.analyze(analyzer)?;

                Self::analyze_condition(condition.as_mut(), analyzer)?;
                Self::analyze_body(body.as_mut(), analyzer)?;

                analyzer.scope.pop();

                Ok(())
            }
            BranchType::Loop { body } => {
                let cnt = analyzer.counter();
                analyzer.scope.push(format!("@l.{}", cnt));

                Self::analyze_body(body.as_mut(), analyzer)?;

                analyzer.scope.pop();

                Ok(())
            }
            BranchType::If { body, condition } => {
                Self::analyze_condition(condition.as_mut(), analyzer)?;
                Self::analyze_body(body.as_mut(), analyzer)?;

                Ok(())
            }
            BranchType::Else { body } => {
                Self::analyze_body(body.as_mut(), analyzer)?;

                Ok(())
            }
        }
    }
}

impl Interupter {
    fn is_in_loop(analyzer: &mut Analyzer) -> bool {
        for s in analyzer.scope.iter().rev() {
            if s.starts_with("@l.") {
                return true;
            }
        }

        false
    }
}

impl Analyze for Interupter {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        let in_loop = Self::is_in_loop(analyzer);

        match &mut self.interupter {
            InterupterType::Break { ret } | InterupterType::Return { ret } => {
                if let Some(expr) = ret {
                    expr.analyze(analyzer)?
                }
            }
            _ => {}
        }

        if !in_loop {
            return Err(Message::new(
                self.location,
                &format!("Unexpected {} statement", self.interupter.to_str()),
            ));
        }

        Ok(())
    }
}
