use std::path::PathBuf;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SerializationOption {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CrateType {
    #[default]
    Bin,
    StaticLib,
    DynamicLib,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    #[default]
    Gcc,
    Clang,
}

#[derive(Default, Debug, Clone)]
pub struct CompileOptions {
    pub crate_name: String,
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub verbose_tokens: bool,
    pub dump_ast_mode: SerializationOption,
    pub allow_variants: bool,
    pub backend: Backend,
    pub crate_type: CrateType,
    pub libraries: Vec<String>,
    pub libraries_paths: Vec<PathBuf>,
}
