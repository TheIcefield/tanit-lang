use expression_utils::{BinaryOperation, UnaryOperation};
use tanitc_attributes::Mutability;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;
use tanitc_ty::Type;

use std::collections::BTreeMap;

pub mod attributes;
pub mod expression_utils;
pub mod variant_utils;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AliasDef {
    pub location: Location,
    pub attributes: attributes::AliasAttributes,
    pub identifier: Ident,
    pub value: TypeSpec,
}

impl From<AliasDef> for Ast {
    fn from(value: AliasDef) -> Self {
        Self::AliasDef(value)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Block {
    pub location: Location,
    pub attributes: attributes::BlockAttributes,
    pub statements: Vec<Ast>,
    pub is_global: bool,
}

impl From<Block> for Ast {
    fn from(value: Block) -> Self {
        Self::Block(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BranchKind {
    Loop { body: Box<Ast> },
    While { body: Box<Ast>, condition: Box<Ast> },
    If { body: Box<Ast>, condition: Box<Ast> },
    Else { body: Box<Ast> },
}

impl BranchKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Loop { .. } => "loop",
            Self::While { .. } => "while",
            Self::If { .. } => "if",
            Self::Else { .. } => "else",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Branch {
    pub location: Location,
    pub kind: BranchKind,
}

impl From<Branch> for Ast {
    fn from(value: Branch) -> Self {
        Self::BranchStmt(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlowKind {
    Return { ret: Option<Box<Ast>> },
    Break { ret: Option<Box<Ast>> },
    Continue,
}

impl ControlFlowKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::Break { .. } => "break",
            Self::Return { .. } => "return",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlFlow {
    pub location: Location,
    pub kind: ControlFlowKind,
}

impl From<ControlFlow> for Ast {
    fn from(value: ControlFlow) -> Self {
        Self::ControlFlow(value)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub location: Location,
    pub attributes: attributes::EnumAttributes,
    pub identifier: Ident,
    pub fields: BTreeMap<Ident, Option<usize>>,
}

impl From<EnumDef> for Ast {
    fn from(value: EnumDef) -> Self {
        Self::EnumDef(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Unary {
        operation: UnaryOperation,
        node: Box<Ast>,
    },
    Binary {
        operation: BinaryOperation,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    Conversion {
        lhs: Box<Ast>,
        ty: TypeSpec,
    },
    Access {
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    Get {
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    Indexing {
        lhs: Box<Ast>,
        index: Box<Ast>,
    },
    Term {
        node: Box<Ast>,
        ty: Type,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub location: Location,
    pub kind: ExpressionKind,
}

impl From<Expression> for Ast {
    fn from(value: Expression) -> Self {
        Self::Expression(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionParam {
    SelfVal(Mutability),
    SelfRef(Mutability),
    SelfPtr(Mutability),
    Common(VariableDef),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FunctionDef {
    pub location: Location,
    pub attributes: attributes::FunctionAttributes,
    pub identifier: Ident,
    pub return_type: TypeSpec,
    pub parameters: Vec<FunctionParam>,
    pub body: Option<Box<Ast>>,
}

impl From<FunctionDef> for Ast {
    fn from(value: FunctionDef) -> Self {
        Self::FuncDef(value)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModuleDef {
    pub location: Location,
    pub attributes: attributes::ModuleAttributes,
    pub identifier: Ident,
    pub is_external: bool,
    pub body: Option<Block>,
}

impl From<ModuleDef> for Ast {
    fn from(value: ModuleDef) -> Self {
        Self::ModuleDef(value)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct FieldInfo {
    pub ty: TypeSpec,
    pub attributes: attributes::FieldAttributes,
}

pub type Fields = BTreeMap<Ident, FieldInfo>;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StructDef {
    pub location: Location,
    pub attributes: attributes::StructAttributes,
    pub identifier: Ident,
    pub fields: Fields,
    pub internals: Vec<Ast>,
}

impl From<StructDef> for Ast {
    fn from(value: StructDef) -> Self {
        Self::StructDef(value)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct UnionDef {
    pub location: Location,
    pub attributes: attributes::UnionAttributes,
    pub identifier: Ident,
    pub fields: Fields,
    pub internals: Vec<Ast>,
}

impl From<UnionDef> for Ast {
    fn from(value: UnionDef) -> Self {
        Self::UnionDef(value)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct ParsedTypeInfo {
    pub mutability: Mutability,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TypeSpec {
    pub location: Location,
    pub info: ParsedTypeInfo,
    pub ty: Type,
}

impl TypeSpec {
    pub fn get_type(&self) -> Type {
        self.ty.clone()
    }

    pub fn get_c_type(&self) -> String {
        self.ty.get_c_type()
    }
}

impl From<TypeSpec> for Ast {
    fn from(value: TypeSpec) -> Self {
        Self::TypeSpec(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallArgKind {
    Notified(Ident, Box<Ast>),
    Positional(usize, Box<Ast>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallArg {
    pub location: Location,
    pub identifier: Option<Ident>,
    pub kind: CallArgKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    Call {
        identifier: Ident,
        arguments: Vec<CallArg>,
    },
    Struct {
        identifier: Ident,
        components: Vec<(Ident, Ast)>,
    },
    Tuple {
        components: Vec<Ast>,
    },
    Array {
        components: Vec<Ast>,
    },
    Identifier(Ident),
    Text(String),
    Integer(usize),
    Decimal(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub location: Location,
    pub kind: ValueKind,
}

impl From<Value> for Ast {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct VariableDef {
    pub location: Location,
    pub attributes: attributes::VariableAttributes,
    pub identifier: Ident,
    pub var_type: TypeSpec,
    pub is_global: bool,
    pub mutability: Mutability,
}

impl From<VariableDef> for Ast {
    fn from(value: VariableDef) -> Self {
        Self::VariableDef(value)
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum VariantField {
    #[default]
    Common,
    StructLike(Fields),
    TupleLike(Vec<TypeSpec>),
}

pub type VariantFields = BTreeMap<Ident, VariantField>;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct VariantDef {
    pub location: Location,
    pub attributes: attributes::VariantAttributes,
    pub identifier: Ident,
    pub fields: VariantFields,
    pub internals: Vec<Ast>,
}

impl From<VariantDef> for Ast {
    fn from(value: VariantDef) -> Self {
        Self::VariantDef(value)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum UseIdentifier {
    #[default]
    BuiltInSelf,
    BuiltInCrate,
    BuiltInSuper,
    BuiltInAll,
    Identifier(Ident),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Use {
    pub location: Location,
    pub identifier: Vec<UseIdentifier>,
}

impl From<Use> for Ast {
    fn from(value: Use) -> Self {
        Self::Use(value)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ExternDef {
    pub location: Location,
    pub abi_name: String,
    pub functions: Vec<FunctionDef>,
}

impl From<ExternDef> for Ast {
    fn from(value: ExternDef) -> Self {
        Self::ExternDef(value)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ImplDef {
    pub location: Location,
    pub attrs: attributes::ImplAttributes,
    pub identifier: Ident,
    pub methods: Vec<FunctionDef>,
}

impl From<ImplDef> for Ast {
    fn from(value: ImplDef) -> Self {
        Self::ImplDef(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
    ModuleDef(ModuleDef),
    StructDef(StructDef),
    UnionDef(UnionDef),
    VariantDef(VariantDef),
    ImplDef(ImplDef),
    EnumDef(EnumDef),
    FuncDef(FunctionDef),
    VariableDef(VariableDef),
    AliasDef(AliasDef),
    ExternDef(ExternDef),
    Expression(Expression),
    BranchStmt(Branch),
    ControlFlow(ControlFlow),
    TypeSpec(TypeSpec),
    Use(Use),
    Block(Block),
    Value(Value),
}

pub trait Visitor {
    fn visit_module_def(&mut self, module_def: &ModuleDef) -> Result<(), Message>;
    fn visit_struct_def(&mut self, struct_def: &StructDef) -> Result<(), Message>;
    fn visit_union_def(&mut self, union_def: &UnionDef) -> Result<(), Message>;
    fn visit_variant_def(&mut self, variant_def: &VariantDef) -> Result<(), Message>;
    fn visit_impl_def(&mut self, impl_def: &ImplDef) -> Result<(), Message>;
    fn visit_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), Message>;
    fn visit_func_def(&mut self, func_def: &FunctionDef) -> Result<(), Message>;
    fn visit_extern_def(&mut self, extern_def: &ExternDef) -> Result<(), Message>;
    fn visit_variable_def(&mut self, var_def: &VariableDef) -> Result<(), Message>;
    fn visit_alias_def(&mut self, alias_def: &AliasDef) -> Result<(), Message>;
    fn visit_expression(&mut self, expr: &Expression) -> Result<(), Message>;
    fn visit_branch(&mut self, branch: &Branch) -> Result<(), Message>;
    fn visit_control_flow(&mut self, cf: &ControlFlow) -> Result<(), Message>;
    fn visit_type_spec(&mut self, type_spec: &TypeSpec) -> Result<(), Message>;
    fn visit_use(&mut self, u: &Use) -> Result<(), Message>;
    fn visit_block(&mut self, block: &Block) -> Result<(), Message>;
    fn visit_value(&mut self, val: &Value) -> Result<(), Message>;
}

pub trait VisitorMut {
    fn visit_module_def(&mut self, module_def: &mut ModuleDef) -> Result<(), Message>;
    fn visit_struct_def(&mut self, struct_def: &mut StructDef) -> Result<(), Message>;
    fn visit_union_def(&mut self, union_def: &mut UnionDef) -> Result<(), Message>;
    fn visit_variant_def(&mut self, variant_def: &mut VariantDef) -> Result<(), Message>;
    fn visit_impl_def(&mut self, impl_def: &mut ImplDef) -> Result<(), Message>;
    fn visit_enum_def(&mut self, enum_def: &mut EnumDef) -> Result<(), Message>;
    fn visit_func_def(&mut self, func_def: &mut FunctionDef) -> Result<(), Message>;
    fn visit_extern_def(&mut self, extern_def: &mut ExternDef) -> Result<(), Message>;
    fn visit_variable_def(&mut self, var_def: &mut VariableDef) -> Result<(), Message>;
    fn visit_alias_def(&mut self, alias_def: &mut AliasDef) -> Result<(), Message>;
    fn visit_expression(&mut self, expr: &mut Expression) -> Result<(), Message>;
    fn visit_branch(&mut self, branch: &mut Branch) -> Result<(), Message>;
    fn visit_control_flow(&mut self, cf: &mut ControlFlow) -> Result<(), Message>;
    fn visit_type_spec(&mut self, type_spec: &mut TypeSpec) -> Result<(), Message>;
    fn visit_use(&mut self, u: &mut Use) -> Result<(), Message>;
    fn visit_block(&mut self, block: &mut Block) -> Result<(), Message>;
    fn visit_value(&mut self, val: &mut Value) -> Result<(), Message>;
}

impl Ast {
    pub fn accept(&self, visitor: &mut dyn Visitor) -> Result<(), Message> {
        match self {
            Self::ModuleDef(node) => visitor.visit_module_def(node),
            Self::StructDef(node) => visitor.visit_struct_def(node),
            Self::UnionDef(node) => visitor.visit_union_def(node),
            Self::VariantDef(node) => visitor.visit_variant_def(node),
            Self::ImplDef(node) => visitor.visit_impl_def(node),
            Self::EnumDef(node) => visitor.visit_enum_def(node),
            Self::FuncDef(node) => visitor.visit_func_def(node),
            Self::VariableDef(node) => visitor.visit_variable_def(node),
            Self::AliasDef(node) => visitor.visit_alias_def(node),
            Self::ExternDef(node) => visitor.visit_extern_def(node),
            Self::Expression(node) => visitor.visit_expression(node),
            Self::BranchStmt(node) => visitor.visit_branch(node),
            Self::ControlFlow(node) => visitor.visit_control_flow(node),
            Self::TypeSpec(node) => visitor.visit_type_spec(node),
            Self::Use(node) => visitor.visit_use(node),
            Self::Block(node) => visitor.visit_block(node),
            Self::Value(node) => visitor.visit_value(node),
        }
    }

    pub fn accept_mut(&mut self, visitor: &mut dyn VisitorMut) -> Result<(), Message> {
        match self {
            Self::ModuleDef(node) => visitor.visit_module_def(node),
            Self::StructDef(node) => visitor.visit_struct_def(node),
            Self::UnionDef(node) => visitor.visit_union_def(node),
            Self::VariantDef(node) => visitor.visit_variant_def(node),
            Self::ImplDef(node) => visitor.visit_impl_def(node),
            Self::EnumDef(node) => visitor.visit_enum_def(node),
            Self::FuncDef(node) => visitor.visit_func_def(node),
            Self::VariableDef(node) => visitor.visit_variable_def(node),
            Self::AliasDef(node) => visitor.visit_alias_def(node),
            Self::ExternDef(node) => visitor.visit_extern_def(node),
            Self::Expression(node) => visitor.visit_expression(node),
            Self::BranchStmt(node) => visitor.visit_branch(node),
            Self::ControlFlow(node) => visitor.visit_control_flow(node),
            Self::TypeSpec(node) => visitor.visit_type_spec(node),
            Self::Use(node) => visitor.visit_use(node),
            Self::Block(node) => visitor.visit_block(node),
            Self::Value(node) => visitor.visit_value(node),
        }
    }

    pub fn location(&self) -> Location {
        match self {
            Self::ModuleDef(node) => node.location,
            Self::StructDef(node) => node.location,
            Self::UnionDef(node) => node.location,
            Self::VariantDef(node) => node.location,
            Self::ImplDef(node) => node.location,
            Self::EnumDef(node) => node.location,
            Self::FuncDef(node) => node.location,
            Self::VariableDef(node) => node.location,
            Self::AliasDef(node) => node.location,
            Self::ExternDef(node) => node.location,
            Self::Expression(node) => node.location,
            Self::BranchStmt(node) => node.location,
            Self::ControlFlow(node) => node.location,
            Self::TypeSpec(node) => node.location,
            Self::Use(node) => node.location,
            Self::Block(node) => node.location,
            Self::Value(node) => node.location,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ModuleDef(_) => "module definition",
            Self::StructDef(_) => "struct definition",
            Self::UnionDef(_) => "union definition",
            Self::VariantDef(_) => "variant definition",
            Self::ImplDef(_) => "impl definition",
            Self::EnumDef(_) => "enum definition",
            Self::FuncDef(_) => "function definition",
            Self::VariableDef(_) => "variable definition",
            Self::AliasDef(_) => "alias definition",
            Self::ExternDef(_) => "extern definition",
            Self::Expression(_) => "expression",
            Self::BranchStmt(_) => "branching",
            Self::ControlFlow(cf) => cf.kind.to_str(),
            Self::TypeSpec(_) => "type specification",
            Self::Use(_) => "using",
            Self::Block(_) => "block",
            Self::Value(_) => "value",
        }
    }

    pub fn apply_attributes(&mut self, attrs: attributes::ParsedAttributes) -> Result<(), Message> {
        let mut visitor = attributes::AttributesApply { attrs };
        self.accept_mut(&mut visitor)
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::Block(Block::default())
    }
}
