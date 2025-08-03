use tanitc_ast::ast::{
    branches::{Branch, BranchKind},
    Ast,
};
use tanitc_messages::Message;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_branch(&mut self, branch: &mut Branch) -> Result<(), Message> {
        let analyze_body = |body: &mut Ast, analyzer: &mut Analyzer| -> Result<(), Message> {
            if let Ast::Block(node) = body {
                for stmt in node.statements.iter_mut() {
                    stmt.accept_mut(analyzer)?;
                }
            }

            Ok(())
        };

        let analyze_condition =
            |condition: &mut Ast, analyzer: &mut Analyzer| -> Result<(), Message> {
                if let Ast::Expression(node) = condition {
                    analyzer.analyze_expression(node)?;
                }

                Ok(())
            };

        let mut scope_info = self.table.get_scope_info();

        match &mut branch.kind {
            BranchKind::While { body, condition } => {
                scope_info.is_in_loop = true;
                self.table.enter_scope(scope_info);

                condition.accept_mut(self)?;

                analyze_condition(condition.as_mut(), self)?;
                analyze_body(body.as_mut(), self)?;

                self.table.exit_scope();

                Ok(())
            }
            BranchKind::Loop { body } => {
                scope_info.is_in_loop = true;
                self.table.enter_scope(scope_info);

                analyze_body(body.as_mut(), self)?;

                self.table.exit_scope();

                Ok(())
            }
            BranchKind::If { body, condition } => {
                analyze_condition(condition.as_mut(), self)?;
                analyze_body(body.as_mut(), self)?;

                Ok(())
            }
            BranchKind::Else { body } => {
                analyze_body(body.as_mut(), self)?;

                Ok(())
            }
        }
    }
}
