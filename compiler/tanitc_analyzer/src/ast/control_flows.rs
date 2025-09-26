use tanitc_ast::ast::control_flows::{ControlFlow, ControlFlowKind};
use tanitc_messages::Message;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_control_flow(&mut self, cf: &mut ControlFlow) -> Result<(), Message> {
        let is_in_func = self.table.get_scope_info().is_in_func;
        let is_in_loop = self.table.get_scope_info().is_in_loop;

        match &mut cf.kind {
            ControlFlowKind::Break { ret } | ControlFlowKind::Return { ret } => {
                if let Some(expr) = ret {
                    expr.accept_mut(self)?;
                }
            }
            _ => {}
        }

        let is_ret = matches!(cf.kind, ControlFlowKind::Return { .. });

        if (!is_ret && !is_in_loop) || (is_ret && !is_in_func) {
            return Err(Message::new(
                &cf.location,
                &format!("Unexpected {} statement", cf.kind.to_str()),
            ));
        }

        Ok(())
    }
}
