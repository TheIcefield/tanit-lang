#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AstSerializeMode {
    #[default]
    None,
    Ron,
    Xml,
    Json,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CrateType {
    #[default]
    StaticLib,
    DynamicLib,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    #[default]
    Gcc,
    Clang,
}

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub input_file: String,
    pub output_file: String,
    pub verbose_tokens: bool,
    pub dump_ast_mode: AstSerializeMode,
    pub dump_symbol_table: bool,
    pub allow_variants: bool,
    pub backend: Backend,
    pub crate_type: CrateType,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            input_file: "main.tt".to_string(),
            output_file: "main.a".to_string(),
            verbose_tokens: false,
            dump_ast_mode: AstSerializeMode::None,
            dump_symbol_table: false,
            allow_variants: false,
            backend: Backend::Gcc,
            crate_type: CrateType::StaticLib,
        }
    }
}
