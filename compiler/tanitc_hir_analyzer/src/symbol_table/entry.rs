use std::collections::BTreeMap;

use tanitc_attributes::Mutability;
use tanitc_hir::hir::type_spec::{FuncType, Type};
use tanitc_ident::Ident;
use tanitc_name::NameSpec;

use crate::symbol_table::table::{Table, TableEntries};

#[derive(Debug, Clone)]
pub struct AliasDefData {
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct ModuleDefData {
    pub name: NameSpec,
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
    pub mutability: Mutability,
    pub is_initialization: bool,
}

#[derive(Debug, Clone)]
pub struct FuncDefData {
    pub name: NameSpec,
    pub ty: FuncType,
    pub is_virtual: bool,
    pub is_inline: bool,
    pub no_return: bool,
}

#[derive(Debug, Clone)]
pub struct StructFieldData {
    pub name: NameSpec,
    pub ty: Type,
}

pub type StructFieldsData = BTreeMap<Ident, StructFieldData>;

#[derive(Debug, Clone)]
pub struct StructDefData {
    pub name: NameSpec,
    pub fields: StructFieldsData,
}

#[derive(Debug, Clone)]
pub struct UnionDefData {
    pub name: NameSpec,
    pub fields: StructFieldsData,
}

#[derive(Debug, Clone)]
pub struct EnumData {
    pub name: NameSpec,
    pub value: usize,
}

pub type EnumDefEntries = std::collections::BTreeMap<Ident, Entry>;

#[derive(Debug, Clone)]
pub struct EnumDefData {
    pub name: NameSpec,
    pub units: EnumDefEntries,
}

#[derive(Debug, Clone)]
pub struct VariantStruct {
    pub name: NameSpec,
    pub fields: StructFieldsData,
}

#[derive(Debug, Clone)]
pub struct VariantTuple {
    pub fields: BTreeMap<usize, StructFieldData>,
}

#[derive(Debug, Clone)]
pub enum VariantKind {
    Struct(VariantStruct),
    Tuple(VariantTuple),
    Enum,
}

#[derive(Debug, Clone)]
pub struct VariantData {
    pub variant_name: NameSpec,    // name specified for a type
    pub variant_unit_id: Ident,    // ident of concrete unit
    pub variant_kind: VariantKind, // kind of concrete unit
    pub variant_kind_num: usize,   // index of concrete unit
}

#[derive(Debug, Clone)]
pub struct VariantDefData {
    pub name: NameSpec,
    pub variants: TableEntries,
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
    pub id: Ident,
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

impl From<VariantData> for SymbolKind {
    fn from(value: VariantData) -> Self {
        Self::Variant(value)
    }
}
