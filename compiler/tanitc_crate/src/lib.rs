use std::{io::Read, path::PathBuf};

use tanitc_ast::program_ctx::ProgramCtx;
use tanitc_ast_lowering::AstLowering;
use tanitc_builder::{build_object_file, link_crate_objects};
use tanitc_hir::hir::Hir;
use tanitc_hir_analyzer::Analyzer;
use tanitc_lexer::Lexer;
use tanitc_options::{CompileOptions, SerializationOption};
use tanitc_parser::Parser;

#[derive(Debug, Clone)]
pub struct Crate {
    name: String,
    initial_path: PathBuf,
    output_path: PathBuf,
    compile_options: CompileOptions,
}

impl Default for Crate {
    fn default() -> Self {
        Self {
            name: "main".to_string(),
            initial_path: PathBuf::from("./main.tt".to_string()),
            output_path: PathBuf::from("./main"),
            compile_options: CompileOptions::default(),
        }
    }
}

impl Crate {
    pub fn new(compile_options: CompileOptions) -> Result<Self, String> {
        let mut c = Self::default();
        c.name = compile_options.crate_name.clone();
        c.output_path = PathBuf::from(format!("{}.tt.c", c.name));
        c.initial_path = PathBuf::from(&compile_options.input_file);
        c.compile_options = compile_options;

        Ok(c)
    }

    pub fn process(&mut self) -> Result<(), String> {
        let ast = self.process_parsing()?;
        if SerializationOption::Enabled == self.compile_options.dump_ast_mode {
            self.serialize_ast(&ast)?;
        }

        let mut hir = self.process_ast_lowering(ast.as_ref())?;

        self.process_analyze(&mut hir)?;
        self.process_codegen(&hir)?;

        self.process_building()?;
        self.process_linkage()?;

        Ok(())
    }
}

impl Crate {
    fn serialize_ast(&mut self, program_ctx: &ProgramCtx) -> Result<(), String> {
        use std::io::Write;

        let file_name = format!("{}.ast.ron", &self.name);
        let mut file = std::fs::File::create(&file_name)
            .map_err(|err| format!("Failed to open \"{file_name}\": {err}"))?;

        writeln!(file, "{program_ctx:#?}")
            .map_err(|err| format!("Failed to serialize AST: {err}"))?;

        Ok(())
    }

    fn process_parsing(&mut self) -> Result<Box<ProgramCtx>, String> {
        let mut file = std::fs::File::open(&self.initial_path)
            .map_err(|err| format!("Failed to open {:?}: {err}", self.initial_path))?;

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .map_err(|err| format!("Failed to read file {:?}: {err}", self.initial_path))?;

        let mut lexer = Lexer::new(buffer.chars().peekable(), &self.initial_path);
        lexer.verbose_tokens = self.compile_options.verbose_tokens;

        let mut parser = Parser::new(lexer);

        let program_ctx = parser.parse_program().map_err(|messages| {
            messages.print_errors();
            "Failed to parse program".to_string()
        })?;

        if parser.messages_ref().has_warnings() {
            parser.messages_ref().print_warnings();
        }

        Ok(program_ctx)
    }

    fn process_ast_lowering(&mut self, program_ctx: &ProgramCtx) -> Result<Box<Hir>, String> {
        let mut lowering = AstLowering::new();

        let hir = lowering.low(program_ctx).map_err(|messages| {
            messages.print_errors();
            "Failed to analyze program".to_string()
        })?;

        if lowering.messages_ref().has_warnings() {
            lowering.messages_ref().print_warnings();
        }

        Ok(hir)
    }

    fn process_analyze(&mut self, hir: &mut Hir) -> Result<(), String> {
        let mut analyzer = Analyzer::with_compile_options(self.compile_options.clone());

        analyzer.analyze_program(hir).map_err(|messages| {
            messages.print_errors();
            "Failed to analyze program".to_string()
        })?;

        if analyzer.messages_ref().has_warnings() {
            analyzer.messages_ref().print_warnings();
        }

        Ok(())
    }

    #[cfg(feature = "backend_C")]
    fn process_codegen(&self, hir: &Hir) -> Result<(), String> {
        let header_name = format!("{}.tt.h", &self.name);
        let mut header_stream = std::fs::File::create(&header_name)
            .map_err(|err| format!("Failed to create \"{header_name}\": {err}"))?;

        let source_name = &self.output_path;
        let mut source_stream = std::fs::File::create(source_name)
            .map_err(|err| format!("Failed to create \"{header_name}\": {err}"))?;

        let mut codegen = tanitc_ir_c::CodeGenStream::with_compile_options(
            &mut header_stream,
            &mut source_stream,
            self.compile_options.clone(),
        );

        codegen.codegen_program(hir)?;

        Ok(())
    }

    fn process_building(&mut self) -> Result<(), String> {
        let built_path = PathBuf::from(format!("{}.o", &self.name));

        build_object_file(&self.output_path, &built_path, &self.compile_options)?;

        Ok(())
    }

    fn process_linkage(&mut self) -> Result<(), String> {
        link_crate_objects(&[&self.output_path], &self.compile_options)?;

        Ok(())
    }
}
