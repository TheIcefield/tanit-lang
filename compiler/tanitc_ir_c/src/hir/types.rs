use tanitc_hir::hir::type_spec::{Type, TypeSpec};

use crate::CodeGenStream;

impl CodeGenStream<'_> {
    pub fn generate_type_spec(&mut self, type_spec: &TypeSpec) -> std::io::Result<()> {
        self.generate_type(&type_spec.ty)
    }

    pub fn generate_type(&mut self, ty: &Type) -> std::io::Result<()> {
        use std::io::Write;

        write!(self, "{}", ty.get_c_type())
    }
}
