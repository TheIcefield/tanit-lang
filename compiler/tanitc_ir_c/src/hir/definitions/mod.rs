use tanitc_hir::hir::definitions::Definition;

use crate::CodeGenStream;

pub(crate) mod aliases;
pub(crate) mod enums;
pub(crate) mod externs;
pub(crate) mod functions;
pub(crate) mod methods;
pub(crate) mod modules;
pub(crate) mod structs;
pub(crate) mod unions;
pub(crate) mod variables;
pub(crate) mod variants;

impl CodeGenStream<'_> {
    pub fn generate_definition(&mut self, node: &Definition) -> std::io::Result<()> {
        match node {
            Definition::Module(node) => self.generate_module_def(node),
            Definition::Struct(node) => self.generate_struct_def(node),
            Definition::Union(node) => self.generate_union_def(node),
            Definition::Variant(node) => self.generate_variant_def(node),
            Definition::Impl(node) => self.generate_impl_def(node),
            Definition::Enum(node) => self.generate_enum_def(node),
            Definition::Func(node) => self.generate_func_def(node, None),
            Definition::Variable(node) => self.generate_variable_def(node),
            Definition::Alias(node) => self.generate_alias_def(node),
            Definition::Extern(node) => self.generate_extern_def(node),
        }
    }
}
