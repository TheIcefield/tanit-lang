use std::collections::BTreeMap;

use tanitc_ident::Ident;
use tanitc_ty::Type;

use crate::table::Table;

#[derive(Debug, Clone)]
pub enum VarStorageType {
    Auto,
    Static,
    Register,
    Extern,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Module {
        table: Box<Table>,
    },
    Var {
        storage: VarStorageType,
        offset: usize,
        size: usize,
    },
    Func {
        parameters: Vec<(Ident, Type)>,
        return_type: Type,
        is_virtual: bool,
        is_inline: bool,
        no_return: bool,
    },
    Struct {
        fields: BTreeMap<Ident, Entry>,
    },
    Union {
        fields: BTreeMap<Ident, Entry>,
    },
    Enum {
        value: usize,
    },
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: Ident,
    pub is_static: bool,
    pub kind: SymbolKind,
}
