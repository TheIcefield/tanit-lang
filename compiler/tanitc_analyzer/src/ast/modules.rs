use tanitc_ast::{ast::modules::ModuleDef, visitor::VisitorMut};
use tanitc_messages::Message;
use tanitc_symbol_table::{
    entry::{Entry, ModuleDefData, SymbolKind},
    table::Table,
};

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_module_def(&mut self, module_def: &mut ModuleDef) -> Result<(), Message> {
        if self.has_symbol(module_def.identifier) {
            return Err(Message::multiple_ids(
                module_def.location,
                module_def.identifier,
            ));
        }

        self.table.insert(Entry {
            name: module_def.identifier,
            is_static: true,
            kind: SymbolKind::from(ModuleDefData {
                table: Box::new(Table::new()),
            }),
        });

        let mut analyzer = Analyzer::with_options(self.compile_options.clone());

        analyzer.visit_block(module_def.body.as_mut())?;
        let entry = self.table.lookup_mut(module_def.identifier).unwrap();
        let SymbolKind::ModuleDef(ref mut data) = &mut entry.kind else {
            unreachable!();
        };

        data.table = analyzer.table;

        Ok(())
    }
}
