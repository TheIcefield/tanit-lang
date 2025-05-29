use tanitc_ident::Ident;
use tanitc_ty::Type;

use crate::scope::Scope;

#[derive(Debug, Clone)]
pub enum SymbolData {
    AliasDef {
        ty: Type,
    },
    ModuleDef,
    StructDef,
    StructField {
        struct_id: Ident,
        ty: Type,
    },
    UnionDef,
    UnionField {
        union_id: Ident,
        ty: Type,
    },
    EnumDef,
    EnumComponent {
        enum_id: Ident,
        val: usize,
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
}

impl Symbol {
    pub fn traverse(&self, stream: &mut dyn std::io::Write) -> std::io::Result<()> {
        match &self.data {
            SymbolData::AliasDef { ty } => write!(stream, "alias definition ({ty})"),
            SymbolData::ModuleDef => write!(stream, "module definition"),
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
            SymbolData::StructDef => write!(stream, "Struct definition"),
            SymbolData::StructField { struct_id, ty } => {
                write!(stream, "field of struct {struct_id}, type: {ty}")
            }
            SymbolData::UnionDef => write!(stream, "Union definition"),
            SymbolData::UnionField { union_id, ty } => {
                write!(stream, "field of union {union_id}, type: {ty}")
            }
            SymbolData::EnumDef => write!(stream, "Enum definition."),
            SymbolData::EnumComponent { enum_id, val } => {
                write!(stream, "{enum_id}::{} = {val}", self.id)
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: Ident,
    pub scope: Scope,
    pub data: SymbolData,
}
