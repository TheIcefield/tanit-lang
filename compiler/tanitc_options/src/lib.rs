#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AstSerializeMode {
    #[default]
    None,
    Ron,
    Xml,
    Json,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    #[default]
    Gcc,
    Clang,
}

#[derive(Clone, Default)]
pub struct CompileOptions {
    pub input_file: String,
    pub output_file: String,
    pub verbose_tokens: bool,
    pub dump_ast_mode: AstSerializeMode,
    pub dump_symbol_table: bool,
    pub allow_variants: bool,
    pub backend: Backend,
}
