use tanitc_hir::{
    hir::definitions::modules::{ModuleDef, ModuleDefBody},
    visitor::VisitorMut,
};
use tanitc_messages::Message;

use crate::{
    symbol_table::{
        entry::{Entry, ModuleDefData, SymbolKind},
        table::Table,
    },
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_module_def(&mut self, module_def: &mut ModuleDef) -> AnalyzeResult<()> {
        if self.has_symbol(module_def.name.id) {
            return Err(Message::multiple_ids(
                module_def.location,
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

        let mut analyzer = Analyzer::new();
        analyzer.set_compile_options(self.compile_options.clone());
        analyzer.table.set_id(joined_id);

        analyzer.analyze_module_def_body(&mut module_def.body)?;

        let entry = self.table.lookup_mut(module_def.name.id).unwrap();
        let SymbolKind::ModuleDef(ref mut data) = &mut entry.kind else {
            unreachable!();
        };

        data.table = analyzer.table;

        Ok(())
    }

    fn analyze_module_def_body(&mut self, body: &mut ModuleDefBody) -> AnalyzeResult<()> {
        match body {
            ModuleDefBody::External(body) => body.accept_mut(self),
            ModuleDefBody::Internal(body) => self.visit_block(body),
        }
    }
}
