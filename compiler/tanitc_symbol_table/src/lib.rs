use tanitc_ident::Ident;

use std::collections::HashMap;

pub mod scope;
pub mod symbol;

pub mod entry;
pub mod table;

use crate::{
    scope::{Scope, ScopeUnitKind},
    symbol::Symbol,
};

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

    pub fn access_symbol(&self, ids: &[Ident], scope: &Scope) -> Vec<&Symbol> {
        let mut ret = self.get_symbols();
        let mut ids_full: Vec<Ident> = vec![];

        for scope_unit in scope.0.iter() {
            if let ScopeUnitKind::Func(_) = scope_unit.kind {
                continue;
            }

            let id = scope_unit.kind.get_id();
            if let Some(id) = id {
                ids_full.push(id);
            }
        }

        for id in ids.iter() {
            ids_full.push(*id);
        }

        for (iter, id) in ids_full.iter().enumerate() {
            if ret.is_empty() {
                break;
            }

            if (iter + 1) >= ids_full.len() {
                // the last iter, check id
                ret.retain(|s| s.id == *id);
                break;
            }

            ret.retain(|s| {
                if s.scope.0.len() <= iter {
                    return false;
                }

                let scope_unit = &s.scope.0[iter];
                let scope_unit_name = scope_unit.kind.get_id();
                scope_unit_name == Some(*id)
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
