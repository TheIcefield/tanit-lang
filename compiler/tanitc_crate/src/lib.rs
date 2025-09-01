use std::path::PathBuf;

use tanitc_analyzer::{self, Analyzer};
use tanitc_ast::ast::Ast;
use tanitc_builder::{build_object_file, link_crate_objects};
use tanitc_codegen::c_generator::{CodeGenMode, CodeGenStream};
use tanitc_lexer::Lexer;
use tanitc_messages::Message;
use tanitc_options::{CompileOptions, CrateType, SerializationOption};
use tanitc_parser::Parser;

#[derive(Debug, Clone)]
pub struct Crate {
    name: String,
    initial_path: PathBuf,
    output_path: PathBuf,
    ast: Option<Ast>,
    compile_options: CompileOptions,
}

impl Default for Crate {
    fn default() -> Self {
        Self {
            name: "main".to_string(),
            initial_path: PathBuf::from("./main.tt".to_string()),
            output_path: PathBuf::from("./main"),
            ast: None,
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
        self.process_parsing()?;
        self.process_analyze()?;

        if SerializationOption::Enabled == self.compile_options.dump_ast_mode {
            self.serialize_ast()?;
        }

        self.process_codegen()?;
        self.process_building()?;
        self.process_linkage()?;

        Ok(())
    }
}

impl Crate {
    fn print_messages(&self, messages: &[Message]) {
        for msg in messages.iter() {
            eprintln!("{msg}");
        }
    }

    fn serialize_ast(&mut self) -> Result<(), String> {
        use std::io::Write;

        let Some(ast) = &mut self.ast else {
            return Err("Serialising requires AST".to_string());
        };

        let mut file = match std::fs::File::create(format!("{}.ast.ron", &self.name)) {
            Ok(file) => file,
            Err(err) => return Err(format!("{err}")),
        };

        if let Err(err) = writeln!(file, "{ast:#?}") {
            return Err(format!("Failed to serialize AST: {err}"));
        }

        Ok(())
    }

    fn parse_program(&mut self, parser: &mut Parser) {
        match parser.parse_global_block() {
            Ok(ast) => self.ast = Some(ast),
            Err(msg) => parser.error(msg),
        }
    }

    fn process_parsing(&mut self) -> Result<(), String> {
        let mut lexer = Lexer::from_file(&self.initial_path)?;
        lexer.verbose_tokens = self.compile_options.verbose_tokens;

        let mut parser = Parser::new(lexer);
        self.parse_program(&mut parser);

        if parser.has_errors() {
            self.print_messages(&parser.get_errors());
            return Err(format!("Failed parsing of {:?}", self.initial_path));
        }

        if parser.has_warnings() {
            self.print_messages(&parser.get_warnings());
        }

        Ok(())
    }

    fn analyze_program(&mut self, analyzer: &mut Analyzer) -> Result<(), String> {
        let Some(ast) = &mut self.ast else {
            return Err("Analysis requires AST".to_string());
        };

        if let Err(err) = ast.accept_mut(analyzer) {
            analyzer.error(err);
        }

        if self.compile_options.crate_type == CrateType::Bin {
            if let Err(err) = analyzer.check_entry_point() {
                analyzer.error(err);
            }
        }

        Ok(())
    }

    fn process_analyze(&mut self) -> Result<(), String> {
        let mut analyzer = Analyzer::with_options(self.compile_options.clone());

        self.analyze_program(&mut analyzer)?;

        if analyzer.has_errors() {
            self.print_messages(&analyzer.get_errors());
            return Err(format!("Failed analysis of {:?}", self.initial_path));
        }

        if analyzer.has_warnings() {
            self.print_messages(&analyzer.get_warnings());
        }

        Ok(())
    }

    fn codegen_program(&self, codegen: &mut CodeGenStream) -> Result<(), String> {
        use std::io::Write;

        let Some(ast) = &self.ast else {
            return Err("Codegen requires AST".to_string());
        };

        codegen.mode = CodeGenMode::SourceOnly;

        if let Err(msg) = writeln!(codegen, "#include \"{}.tt.h\"\n", &self.name) {
            return Err(format!("{msg}"));
        };

        codegen.mode = CodeGenMode::Unset;

        if let Err(err) = ast.accept(codegen) {
            return Err(format!("Error: {err}"));
        }

        Ok(())
    }

    fn process_codegen(&self) -> Result<(), String> {
        let Ok(mut header_stream) = std::fs::File::create(format!("{}.tt.h", &self.name)) else {
            return Err("Error: can't create file for header stream".to_string());
        };

        let Ok(mut source_stream) = std::fs::File::create(&self.output_path) else {
            return Err("Error: can't create file for source stream".to_string());
        };

        let Ok(mut codegen) = CodeGenStream::new(&mut header_stream, &mut source_stream) else {
            return Err("Error: can't create codegen writer".to_string());
        };

        self.codegen_program(&mut codegen)?;

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
