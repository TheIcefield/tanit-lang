use std::collections::BTreeMap;

use tanitc_ident::Ident;
use tanitc_ty::Type;

use crate::scope::Scope;

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
    EnumDef,
    EnumComponent {
        enum_id: Ident,
        val: usize,
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

impl Symbol {
    pub fn traverse(&self, stream: &mut dyn std::io::Write) -> std::io::Result<()> {
        match &self.data {
            SymbolData::ModuleDef { full_name } => write!(stream, "{:?}", full_name),
            SymbolData::FunctionDef {
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
            SymbolData::StructDef { components } => {
                write!(stream, "Struct definition: {{{:?}}}", components)
            }
            SymbolData::EnumDef => write!(stream, "Enum definition."),
            SymbolData::EnumComponent { enum_id, val } => {
                write!(stream, "{enum_id}::{} = {val}", self.id)
            }
            SymbolData::VariantDef { components } => {
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
            SymbolData::VariableDef {
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
            SymbolData::Type => write!(stream, "type"),
        }
    }
}

#[derive(Clone)]
pub struct Symbol {
    pub id: Ident,
    pub scope: Scope,
    pub data: SymbolData,
}
