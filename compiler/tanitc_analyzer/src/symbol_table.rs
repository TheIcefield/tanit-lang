use super::scope::Scope;

use tanitc_ident::Ident;
use tanitc_ty::Type;

use std::collections::{BTreeMap, HashMap};
use std::io::Write;

#[derive(Clone, PartialEq, Default)]
pub enum VariantFieldData {
    #[default]
    Common,
    StructLike(BTreeMap<Ident, Type>),
    TupleLike(Vec<Type>),
}

#[derive(Clone)]
pub enum SymbolData {
    ModuleDef {
        full_name: Vec<Ident>,
    },
    StructDef {
        components: Vec<Type>,
    },
    EnumDef {
        components: Vec<(Ident, usize)>,
    },
    VariantDef {
        components: Vec<VariantFieldData>,
    },
    FunctionDef {
        parameters: Vec<(Ident, Type)>,
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
    pub fn traverse(&self, stream: &mut dyn std::io::Write) -> std::io::Result<()> {
        match self {
            Self::ModuleDef { full_name } => write!(stream, "{:?}", full_name),
            Self::FunctionDef {
                parameters,
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

                for param in parameters.iter() {
                    write!(stream, "{}:{} ", param.0, param.1)?;
                }

                write!(stream, ") -> {}", return_type)
            }
            Self::StructDef { components } => {
                write!(stream, "Struct definition: {{{:?}}}", components)
            }
            Self::EnumDef { components } => {
                write!(stream, "Enum definition: ")?;
                for comp in components.iter() {
                    write!(stream, "{}:{} ", comp.0, comp.1)?;
                }
                Ok(())
            }
            Self::VariantDef { components } => {
                write!(stream, "Enum definition: <")?;

                for comp in components.iter() {
                    match comp {
                        VariantFieldData::Common => write!(stream, "common ")?,
                        VariantFieldData::TupleLike(t) => {
                            write!(stream, "( ")?;
                            for tc in t.iter() {
                                write!(stream, "{} ", tc)?;
                            }
                            write!(stream, ")")?;
                        }
                        VariantFieldData::StructLike(s) => {
                            write!(stream, "{{ ")?;
                            for sc in s.iter() {
                                write!(stream, "{} ", sc.1)?;
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

    pub fn insert(&mut self, id: Ident, symbol: Symbol) {
        self.table.entry(id).or_default();

        if let Some(ss) = self.table.get_mut(&id) {
            ss.push(symbol)
        }
    }

    pub fn traverse(&self, stream: &mut std::fs::File) -> std::io::Result<()> {
        for (identifier, ss) in self.table.iter() {
            let s = identifier.to_string();
            writeln!(stream, "Identifier: {s}#{identifier}:")?;

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
