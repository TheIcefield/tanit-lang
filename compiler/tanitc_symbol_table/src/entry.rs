use std::collections::BTreeMap;

use tanitc_ident::Ident;
use tanitc_ty::Type;

use crate::table::Table;

#[derive(Debug, Clone)]
pub struct AliasDefData {
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct ModuleDefData {
    pub table: Box<Table>,
}

#[derive(Debug, Clone)]
pub enum VarStorageType {
    Auto,
    Static,
    Register,
    Extern,
}

#[derive(Debug, Clone)]
pub struct VarDefData {
    pub storage: VarStorageType,
    pub var_type: Type,
    pub is_mutable: bool,
    pub is_initialization: bool,
}

#[derive(Debug, Clone)]
pub struct FuncDefData {
    pub parameters: Vec<(Ident, Type)>,
    pub return_type: Type,
    pub is_virtual: bool,
    pub is_inline: bool,
    pub no_return: bool,
}

#[derive(Debug, Clone)]
pub struct StructFieldData {
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct StructDefData {
    pub fields: BTreeMap<Ident, StructFieldData>,
}

#[derive(Debug, Clone)]
pub struct UnionDefData {
    pub fields: BTreeMap<Ident, StructFieldData>,
}

#[derive(Debug, Clone)]
pub struct EnumData {
    pub enum_name: Ident,
    pub value: usize,
}

#[derive(Debug, Clone)]
pub struct EnumDefData {
    pub enums: BTreeMap<Ident, Entry /* Only Enum */>,
}

#[derive(Debug, Clone)]
pub struct VariantStructKind {
    pub fields: BTreeMap<Ident, StructFieldData>,
}

#[derive(Debug, Clone)]
pub struct VariantTupleKind {
    pub fields: BTreeMap<usize, StructFieldData>,
}

#[derive(Debug, Clone)]
pub enum VariantKind {
    VariantStructKind(VariantStructKind),
    VariantTupleKind(VariantTupleKind),
    EnumKind,
}

#[derive(Debug, Clone)]
pub struct VariantData {
    pub variant_name: Ident,
    pub variant_kind: VariantKind,
    pub variant_kind_num: usize,
}

#[derive(Debug, Clone)]
pub struct VariantDefData {
    pub variants: BTreeMap<Ident, Entry>,
}

#[derive(Default, Debug, Clone)]
pub enum SymbolKind {
    #[default]
    None,
    AliasDef(AliasDefData),
    ModuleDef(ModuleDefData),
    VarDef(VarDefData),
    FuncDef(FuncDefData),
    StructDef(StructDefData),
    UnionDef(UnionDefData),
    EnumDef(EnumDefData),
    Enum(EnumData),
    VariantDef(VariantDefData),
    Variant(VariantData),
}

#[derive(Default, Debug, Clone)]
pub struct Entry {
    pub name: Ident,
    pub is_static: bool,
    pub kind: SymbolKind,
}

impl From<AliasDefData> for SymbolKind {
    fn from(value: AliasDefData) -> Self {
        Self::AliasDef(value)
    }
}

impl From<ModuleDefData> for SymbolKind {
    fn from(value: ModuleDefData) -> Self {
        Self::ModuleDef(value)
    }
}

impl From<VarDefData> for SymbolKind {
    fn from(value: VarDefData) -> Self {
        Self::VarDef(value)
    }
}

impl From<FuncDefData> for SymbolKind {
    fn from(value: FuncDefData) -> Self {
        Self::FuncDef(value)
    }
}

impl From<StructDefData> for SymbolKind {
    fn from(value: StructDefData) -> Self {
        Self::StructDef(value)
    }
}

impl From<UnionDefData> for SymbolKind {
    fn from(value: UnionDefData) -> Self {
        Self::UnionDef(value)
    }
}

impl From<EnumDefData> for SymbolKind {
    fn from(value: EnumDefData) -> Self {
        Self::EnumDef(value)
    }
}

impl From<EnumData> for SymbolKind {
    fn from(value: EnumData) -> Self {
        Self::Enum(value)
    }
}

impl From<VariantDefData> for SymbolKind {
    fn from(value: VariantDefData) -> Self {
        Self::VariantDef(value)
    }
}
