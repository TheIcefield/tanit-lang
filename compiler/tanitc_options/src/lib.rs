#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AstSerializeMode {
    #[default]
    Ron,
    Xml,
    Json,
}

#[derive(Clone, Copy, Default)]
pub struct CompileOptions {
    pub verbose_tokens: bool,
    pub dump_ast_mode: Option<AstSerializeMode>,
    pub dump_symbol_table: bool,
    pub allow_variants: bool,
}
