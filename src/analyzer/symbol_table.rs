use super::scope::Scope;
use crate::ast::{identifiers::Identifier, types::Type, variants::VariantField};

use std::collections::HashMap;
use std::io::Write;

#[derive(Clone)]
pub enum SymbolData {
    ModuleDef {
        full_name: Vec<String>,
    },
    StructDef {
        components: Vec<Type>,
    },
    VariantDef {
        components: Vec<VariantField>,
    },
    FunctionDef {
        args: Vec<(String, Type)>,
        return_type: Type,
        is_declaration: bool,
    },
    VariableDef {
        var_type: Type,
        is_mutable: bool,
        is_initialization: bool,
    },
    Type,
}

impl SymbolData {
    pub fn traverse(&self, stream: &mut std::fs::File) -> std::io::Result<()> {
        match self {
            Self::ModuleDef { full_name } => write!(stream, "{:?}", full_name),

            Self::FunctionDef {
                args,
                return_type,
                is_declaration,
            } => {
                write!(
                    stream,
                    "Function {}: ( ",
                    if *is_declaration {
                        "declaration"
                    } else {
                        "definition"
                    }
                )?;

                for arg in args.iter() {
                    write!(stream, "{}:{} ", arg.0, arg.1)?;
                }

                write!(stream, ") -> {}", return_type)
            }

            Self::StructDef { components } => {
                write!(stream, "Struct definition: {{{:?}}}", components)
            }

            Self::VariantDef { components } => {
                write!(stream, "Enum definition: <")?;

                for comp in components.iter() {
                    match comp {
                        VariantField::Common => write!(stream, "common ")?,
                        VariantField::TupleLike(t) => {
                            write!(stream, "( ")?;
                            for tc in t.iter() {
                                write!(stream, "{} ", *tc)?;
                            }
                            write!(stream, ")")?;
                        }
                        VariantField::StructLike(s) => {
                            write!(stream, "{{ ")?;
                            for sc in s.iter() {
                                write!(stream, "{} ", *sc.1)?;
                            }
                            write!(stream, "}}")?;
                        }
                    }
                }

                write!(stream, ">")
            }

            Self::VariableDef {
                var_type,
                is_mutable,
                is_initialization,
            } => write!(
                stream,
                "Variable {}: {} {}",
                if *is_initialization {
                    "initialization"
                } else {
                    "definition"
                },
                if *is_mutable { "mut" } else { "" },
                var_type
            ),

            Self::Type => write!(stream, "type"),
        }
    }
}

#[derive(Clone)]
pub struct Symbol {
    pub scope: Scope,
    pub data: SymbolData,
}

#[derive(Clone)]
pub struct SymbolTable {
    table: HashMap<Identifier, Vec<Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn get(&self, id: &Identifier) -> Option<&Vec<Symbol>> {
        self.table.get(id)
    }

    pub fn get_mut(&mut self, id: &Identifier) -> Option<&mut Vec<Symbol>> {
        self.table.get_mut(id)
    }

    pub fn insert(&mut self, id: &Identifier, symbol: Symbol) {
        if !self.table.contains_key(id) {
            self.table.insert(id.clone(), Vec::new());
        }

        if let Some(ss) = self.table.get_mut(id) {
            ss.push(symbol)
        }
    }

    pub fn traverse(&self, stream: &mut std::fs::File) -> std::io::Result<()> {
        for (identifier, ss) in self.table.iter() {
            writeln!(stream, "Identifier: \"{}\":", identifier)?;

            for s in ss.iter() {
                write!(stream, "+--- ")?;

                s.data.traverse(stream)?;

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
