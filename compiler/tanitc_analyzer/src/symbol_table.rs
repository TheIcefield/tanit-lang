use tanitc_ident::Ident;

use std::collections::HashMap;

use crate::symbol::Symbol;

#[derive(Clone)]
pub struct SymbolTable {
    table: HashMap<Ident, Vec<Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn get(&self, id: &Ident) -> Option<&Vec<Symbol>> {
        self.table.get(id)
    }

    pub fn get_mut(&mut self, id: &Ident) -> Option<&mut Vec<Symbol>> {
        self.table.get_mut(id)
    }

    pub fn get_symbols(&self) -> Vec<&Symbol> {
        let mut res: Vec<&Symbol> = vec![];
        for ss in self.table.iter() {
            for s in ss.1.iter() {
                res.push(s);
            }
        }
        res
    }

    pub fn access_symbol(&self, ids: &[Ident]) -> Vec<&Symbol> {
        let mut ret = self.get_symbols();

        for (iter, id) in ids.iter().enumerate() {
            if (iter + 1) >= ids.len() {
                // the last iter, check id
                ret.retain(|s| s.id == *id);
                break;
            }
            ret.retain(|s| {
                s.scope.0.len() > (iter + 1) && s.scope.0[iter + 1].get_id() == Some(*id)
            });
        }

        ret
    }

    pub fn insert(&mut self, id: Ident, symbol: Symbol) {
        self.table.entry(id).or_default();

        if let Some(ss) = self.table.get_mut(&id) {
            ss.push(symbol)
        }
    }

    pub fn traverse(&self, stream: &mut dyn std::io::Write) -> std::io::Result<()> {
        for (identifier, ss) in self.table.iter() {
            writeln!(stream, "Identifier: {identifier}#{}:", identifier.index())?;

            for s in ss.iter() {
                write!(stream, "+--- ")?;

                s.traverse(stream)?;

                writeln!(stream, " at {:?}", s.scope)?;
            }
        }
        Ok(())
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
