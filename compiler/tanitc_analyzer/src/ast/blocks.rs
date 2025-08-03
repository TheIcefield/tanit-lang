use tanitc_ast::ast::{blocks::Block, Ast};
use tanitc_messages::Message;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_block(&mut self, block: &mut Block) -> Result<(), Message> {
        if block.is_global {
            self.analyze_global_block(block)?;
        } else {
            self.analyze_local_block(block)?;
        }

        Ok(())
    }

    fn analyze_global_block(&mut self, block: &mut Block) -> Result<(), Message> {
        for n in block.statements.iter_mut() {
            let is_denied = matches!(
                n,
                Ast::ControlFlow(_)
                    | Ast::Block(_)
                    | Ast::Value(_)
                    | Ast::BranchStmt(_)
                    | Ast::Expression(_)
                    | Ast::TypeSpec(_)
            );

            if is_denied {
                self.error(Message {
                    location: n.location(),
                    text: format!("Node \"{}\" is not allowed in global scope", n.name()),
                });

                continue;
            }

            if let Err(err) = n.accept_mut(self) {
                self.error(err);
            }
        }

        Ok(())
    }

    fn analyze_local_block(&mut self, block: &mut Block) -> Result<(), Message> {
        let mut scope_info = self.table.get_scope_info();
        scope_info.safety = block.attributes.safety;

        self.table.enter_scope(scope_info);

        for n in block.statements.iter_mut() {
            let is_denied = matches!(
                n,
                Ast::StructDef(_)
                    | Ast::UnionDef(_)
                    | Ast::VariantDef(_)
                    | Ast::FuncDef(_)
                    | Ast::AliasDef(_)
                    | Ast::EnumDef(_)
            );

            if is_denied {
                self.error(Message {
                    location: n.location(),
                    text: format!("Node \"{}\" is not allowed in local scope", n.name()),
                });

                continue;
            }

            if let Err(err) = n.accept_mut(self) {
                self.error(err);
            }
        }

        self.table.exit_scope();

        Ok(())
    }
}
