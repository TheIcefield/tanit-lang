use tanitc_ast::{ast::modules::ModuleDef, visitor::VisitorMut};
use tanitc_messages::Message;
use tanitc_symbol_table::{
    entry::{Entry, ModuleDefData, SymbolKind},
    table::Table,
};

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_module_def(&mut self, module_def: &mut ModuleDef) -> Result<(), Message> {
        if self.has_symbol(module_def.name.id) {
            return Err(Message::multiple_ids(
                &module_def.location,
                module_def.name.id,
            ));
        }

        module_def.name.prefix = self.table.get_id();

        self.table.insert(Entry {
            name: module_def.name.id,
            is_static: true,
            kind: SymbolKind::from(ModuleDefData {
                table: Box::new(Table::new()),
            }),
        });

        module_def.name.prefix = self.table.get_id();

        let joined_id = self.table.get_joined_id(module_def.name.id);

        let mut analyzer = Analyzer::with_options(self.compile_options.clone());
        analyzer.table.set_id(joined_id);

        analyzer.visit_block(module_def.body.as_mut())?;
        let entry = self.table.lookup_mut(module_def.name.id).unwrap();
        let SymbolKind::ModuleDef(ref mut data) = &mut entry.kind else {
            unreachable!();
        };

        data.table = analyzer.table;

        Ok(())
    }
}
