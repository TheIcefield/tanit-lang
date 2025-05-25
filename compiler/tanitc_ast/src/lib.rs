use attributes::Attributes;
use expression_utils::{BinaryOperation, UnaryOperation};
use tanitc_ident::Ident;
use tanitc_lexer::{location::Location, token::Lexem};
use tanitc_messages::Message;
use tanitc_ty::Type;

use std::collections::BTreeMap;

pub mod attributes;
pub mod expression_utils;
pub mod variant_utils;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AliasDef {
    pub location: Location,
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
    pub attrs: Attributes,
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
            ControlFlowKind::Continue => "continue",
            ControlFlowKind::Break { .. } => "break",
            ControlFlowKind::Return { .. } => "return",
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FunctionDef {
    pub location: Location,
    pub attrs: Attributes,
    pub identifier: Ident,
    pub return_type: TypeSpec,
    pub parameters: Vec<Ast>,
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
    pub attrs: Attributes,
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
pub struct StructDef {
    pub location: Location,
    pub identifier: Ident,
    pub fields: BTreeMap<Ident, TypeSpec>,
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
    pub identifier: Ident,
    pub fields: BTreeMap<Ident, TypeSpec>,
    pub internals: Vec<Ast>,
}

impl From<UnionDef> for Ast {
    fn from(value: UnionDef) -> Self {
        Self::UnionDef(value)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct TypeInfo {
    pub is_mut: bool,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TypeSpec {
    pub location: Location,
    pub info: TypeInfo,
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

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDef {
    pub location: Location,
    pub identifier: Ident,
    pub var_type: TypeSpec,
    pub is_global: bool,
    pub is_mutable: bool,
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
    StructLike(BTreeMap<Ident, TypeSpec>),
    TupleLike(Vec<TypeSpec>),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct VariantDef {
    pub location: Location,
    pub identifier: Ident,
    pub fields: BTreeMap<Ident, VariantField>,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
    ModuleDef(ModuleDef),
    StructDef(StructDef),
    UnionDef(UnionDef),
    VariantDef(VariantDef),
    EnumDef(EnumDef),
    FuncDef(FunctionDef),
    VariableDef(VariableDef),
    AliasDef(AliasDef),
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
    fn visit_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), Message>;
    fn visit_func_def(&mut self, func_def: &FunctionDef) -> Result<(), Message>;
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
    fn visit_enum_def(&mut self, enum_def: &mut EnumDef) -> Result<(), Message>;
    fn visit_func_def(&mut self, func_def: &mut FunctionDef) -> Result<(), Message>;
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
            Ast::ModuleDef(node) => visitor.visit_module_def(node),
            Ast::StructDef(node) => visitor.visit_struct_def(node),
            Ast::UnionDef(node) => visitor.visit_union_def(node),
            Ast::VariantDef(node) => visitor.visit_variant_def(node),
            Ast::EnumDef(node) => visitor.visit_enum_def(node),
            Ast::FuncDef(node) => visitor.visit_func_def(node),
            Ast::VariableDef(node) => visitor.visit_variable_def(node),
            Ast::AliasDef(node) => visitor.visit_alias_def(node),
            Ast::Expression(node) => visitor.visit_expression(node),
            Ast::BranchStmt(node) => visitor.visit_branch(node),
            Ast::ControlFlow(node) => visitor.visit_control_flow(node),
            Ast::TypeSpec(node) => visitor.visit_type_spec(node),
            Ast::Use(node) => visitor.visit_use(node),
            Ast::Block(node) => visitor.visit_block(node),
            Ast::Value(node) => visitor.visit_value(node),
        }
    }

    pub fn accept_mut(&mut self, visitor: &mut dyn VisitorMut) -> Result<(), Message> {
        match self {
            Ast::ModuleDef(node) => visitor.visit_module_def(node),
            Ast::StructDef(node) => visitor.visit_struct_def(node),
            Ast::UnionDef(node) => visitor.visit_union_def(node),
            Ast::VariantDef(node) => visitor.visit_variant_def(node),
            Ast::EnumDef(node) => visitor.visit_enum_def(node),
            Ast::FuncDef(node) => visitor.visit_func_def(node),
            Ast::VariableDef(node) => visitor.visit_variable_def(node),
            Ast::AliasDef(node) => visitor.visit_alias_def(node),
            Ast::Expression(node) => visitor.visit_expression(node),
            Ast::BranchStmt(node) => visitor.visit_branch(node),
            Ast::ControlFlow(node) => visitor.visit_control_flow(node),
            Ast::TypeSpec(node) => visitor.visit_type_spec(node),
            Ast::Use(node) => visitor.visit_use(node),
            Ast::Block(node) => visitor.visit_block(node),
            Ast::Value(node) => visitor.visit_value(node),
        }
    }

    pub fn location(&self) -> Location {
        match self {
            Self::ModuleDef(node) => node.location,
            Self::StructDef(node) => node.location,
            Self::UnionDef(node) => node.location,
            Self::VariantDef(node) => node.location,
            Self::EnumDef(node) => node.location,
            Self::FuncDef(node) => node.location,
            Self::VariableDef(node) => node.location,
            Self::AliasDef(node) => node.location,
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
            Ast::ModuleDef(_) => "module definition",
            Ast::StructDef(_) => "struct definition",
            Ast::UnionDef(_) => "union definition",
            Ast::VariantDef(_) => "variant definition",
            Ast::EnumDef(_) => "enum definition",
            Ast::FuncDef(_) => "function definition",
            Ast::VariableDef(_) => "variable definition",
            Ast::AliasDef(_) => "alias definition",
            Ast::Expression(_) => "expression",
            Ast::BranchStmt(_) => "branching",
            Ast::ControlFlow(cf) => cf.kind.to_str(),
            Ast::TypeSpec(_) => "type specification",
            Ast::Use(_) => "using",
            Ast::Block(_) => "block",
            Ast::Value(_) => "value",
        }
    }

    pub fn apply_attributes(&mut self, attrs: attributes::Attributes) -> Result<(), Message> {
        let mut visitor = attributes::AttributesApply { attrs };
        self.accept_mut(&mut visitor)
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::Block(Block::default())
    }
}

impl ExpressionKind {
    pub fn new_unary(operator: Lexem, operand: Box<Ast>) -> Result<Self, Message> {
        let operation = match UnaryOperation::try_from(operator) {
            Ok(operation) => operation,
            Err(err) => return Err(Message::new(operand.location(), &err)),
        };

        Ok(Self::Unary {
            operation,
            node: operand,
        })
    }

    pub fn new_binary(operator: Lexem, lhs: Box<Ast>, rhs: Box<Ast>) -> Result<Self, Message> {
        let operation = match BinaryOperation::try_from(operator) {
            Ok(operation) => operation,
            Err(err) => return Err(Message::new(lhs.location(), &err)),
        };

        Ok(match operation {
            BinaryOperation::Access => Self::Access { lhs, rhs },
            // BinaryOperation::Get => Self::Get { lhs, rhs },
            _ => Self::Binary {
                operation,
                lhs,
                rhs,
            },
        })
    }
}
