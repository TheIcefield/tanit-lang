#[derive(Clone, Copy, Default)]
pub struct CompileOptions {
    pub verbose_tokens: bool,
    pub dump_ast: bool,
    pub dump_symbol_table: bool,
    pub allow_variants: bool,
}
