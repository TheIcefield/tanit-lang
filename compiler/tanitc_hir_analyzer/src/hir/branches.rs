use tanitc_hir::hir::branches::{Branch, Else, ElseBody, If, Loop, While};
use tanitc_messages::Message;

use crate::{AnalyzeResult, Analyzer};

impl Analyzer {
    pub(crate) fn analyze_branch(&mut self, branch: &mut Branch) -> Result<(), Message> {
        match branch {
            Branch::While(while_branch) => self.analyze_while_branch(while_branch),
            Branch::Loop(loop_branch) => self.analyze_loop_branch(loop_branch),
            Branch::If(if_branch) => self.analyze_if_branch(if_branch),
            Branch::Else(else_branch) => self.analyze_else_branch(else_branch),
        }
    }

    fn analyze_if_branch(&mut self, if_branch: &mut If) -> AnalyzeResult<()> {
        self.analyze_expression(&mut if_branch.condition)?;
        self.analyze_block(&mut if_branch.body)?;

        Ok(())
    }

    fn analyze_else_branch(&mut self, else_branch: &mut Else) -> AnalyzeResult<()> {
        match &mut else_branch.body {
            ElseBody::If(else_if_body) => self.analyze_if_branch(else_if_body),
            ElseBody::Block(else_block) => self.analyze_block(else_block),
        }
    }

    fn analyze_loop_branch(&mut self, loop_branch: &mut Loop) -> AnalyzeResult<()> {
        let mut scope_info = self.table.get_scope_info();

        scope_info.is_in_loop = true;
        self.table.enter_scope(scope_info);

        self.analyze_block(&mut loop_branch.body)?;

        self.table.exit_scope();

        Ok(())
    }

    fn analyze_while_branch(&mut self, while_branch: &mut While) -> AnalyzeResult<()> {
        let mut scope_info = self.table.get_scope_info();

        scope_info.is_in_loop = true;
        self.table.enter_scope(scope_info);

        self.analyze_expression(&mut while_branch.condition)?;
        self.analyze_block(&mut while_branch.body)?;

        self.table.exit_scope();

        Ok(())
    }
}
