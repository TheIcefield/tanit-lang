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

#[derive(Debug, Clone)]
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

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            crate_name: "".to_string(),
            input_file: PathBuf::from(""),
            output_file: PathBuf::from(""),
            verbose_tokens: false,
            dump_ast_mode: SerializationOption::default(),
            allow_variants: false,
            backend: Backend::default(),
            crate_type: CrateType::default(),
            libraries: Vec::new(),
            libraries_paths: Vec::new(),
        }
    }
}
