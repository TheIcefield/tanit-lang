use std::sync::Mutex;

use crate::analyzer::{self, symbol_table::SymbolTable};
use crate::ast::Ast;
use crate::codegen::{CodeGenMode, CodeGenStream};
use crate::messages;
use crate::parser;
use crate::serializer::XmlWriter;

use tanitc_lexer::Lexer;

use lazy_static::lazy_static;

#[derive(Clone, Copy, Default)]
pub struct CompileOptions {
    pub verbose_tokens: bool,
    pub dump_ast: bool,
    pub dump_symbol_table: bool,
}

#[derive(Default)]
enum UnitProcessState {
    #[default]
    NotProcessed,
    Parsed,
    Analyzed,
    Processed,
}

pub struct Unit {
    name: String,
    path: String,
    process_state: UnitProcessState,
    ast: Option<Ast>,
    symbol_table: Option<SymbolTable>,
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            name: "main".to_string(),
            path: "./main.tt".to_string(),
            process_state: UnitProcessState::default(),
            ast: None,
            symbol_table: None,
        }
    }
}

impl Unit {
    pub fn builder() -> UnitBuilder {
        UnitBuilder::default()
    }

    pub fn from_file(path: String) -> Self {
        let name = path
            .chars()
            .rev()
            .collect::<String>()
            .splitn(2, '/')
            .collect::<Vec<&str>>()[0]
            .chars()
            .rev()
            .collect::<String>()
            .to_string();

        Self {
            name,
            path,
            ..Default::default()
        }
    }

    pub fn process_parsing(&mut self) -> Result<(), &'static str> {
        if !matches!(self.process_state, UnitProcessState::NotProcessed) {
            return Err("Error: expected \"NotProcessed\" state");
        }

        print!("Parsing: \"{}\"... ", &self.path);

        let mut lexer = Lexer::from_file(&self.path)?;
        lexer.verbose_tokens = get_compile_options().verbose_tokens;

        let mut parser = parser::Parser::new(lexer);

        self.ast = parser.parse();

        if parser.has_errors() || self.ast.is_none() {
            messages::print_messages(&parser.get_errors());
            return Err("Parse errors occured");
        }

        if parser.has_warnings() {
            messages::print_messages(&parser.get_warnings());
        }

        self.process_state = UnitProcessState::Parsed;

        println!("OK!");

        Ok(())
    }

    pub fn process_analyze(&mut self) -> Result<(), &'static str> {
        if !matches!(self.process_state, UnitProcessState::Parsed) {
            return Err("Error: expected \"Parsed\" stated");
        }

        if self.ast.is_none() {
            return Err("Error: required \"AST\"");
        }

        print!("Analyzing: \"{}\"... ", &self.path);

        let mut analyzer = analyzer::Analyzer::new();

        self.symbol_table = analyzer.analyze(self.ast.as_mut().unwrap());

        if analyzer.has_errors() || self.symbol_table.is_none() {
            messages::print_messages(&analyzer.get_errors());
            return Err("Analyze errors occured");
        }

        if analyzer.has_warnings() {
            messages::print_messages(&analyzer.get_warnings());
        }

        self.process_state = UnitProcessState::Analyzed;

        println!("OK!");

        let compile_options = get_compile_options();

        if compile_options.dump_ast {
            print!("Serializing AST: \"{}\"... ", &self.path);
            self.serialize_ast(self.ast.as_ref().unwrap());
            println!("OK!");
        }

        if compile_options.dump_symbol_table {
            print!("Serializing symbol table: \"{}\"... ", &self.path);
            self.serialize_symbol_table(self.symbol_table.as_ref().unwrap());
            println!("OK!");
        }

        Ok(())
    }

    pub fn process_codegen(&mut self) -> Result<(), &'static str> {
        use std::io::Write;

        if !matches!(self.process_state, UnitProcessState::Analyzed) {
            return Err("Error: expected \"Parsed\" stated");
        }

        if self.ast.is_none() {
            return Err("Error: missing required \"AST\"");
        }

        if self.symbol_table.is_none() {
            return Err("Error: missing required \"Symbol table\"");
        }

        print!("Building \"{}\"... ", &self.path);

        let mut header_stream = std::fs::File::create(format!("{}.tt.h", &self.name))
            .expect("Error: can't create file for header stream");
        let mut source_stream = std::fs::File::create(format!("{}.tt.c", &self.name))
            .expect("Error: can't create file for source stream");

        let mut writer = CodeGenStream::new(&mut header_stream, &mut source_stream)
            .expect("Error: can't create codegen writer");

        let old_mode = writer.mode;
        writer.mode = CodeGenMode::SourceOnly;

        writeln!(writer, "#include \"{}.tt.h\"\n", &self.name).unwrap();

        writer.mode = old_mode;

        if let Err(err) = self.ast.as_ref().unwrap().codegen(&mut writer) {
            eprintln!("Error: {}", err);
        } else {
            self.process_state = UnitProcessState::Processed;
            println!("OK!");
        }

        Ok(())
    }

    pub fn process() -> Result<(), &'static str> {
        for unit in UNITS.lock().unwrap().iter_mut() {
            if matches!(unit.process_state, UnitProcessState::NotProcessed) {
                unit.process_parsing()?;
            }
        }

        for unit in UNITS.lock().unwrap().iter_mut() {
            if matches!(unit.process_state, UnitProcessState::Parsed) {
                unit.process_analyze()?;
            }
        }

        for unit in UNITS.lock().unwrap().iter_mut() {
            if matches!(unit.process_state, UnitProcessState::Analyzed) {
                unit.process_codegen()?;
            }
        }

        Ok(())
    }

    fn serialize_ast(&self, ast: &Ast) {
        let mut file = std::fs::File::create(format!("{}.ast.xml", &self.name))
            .expect("Error: can't create file for dumping AST");

        let mut writer = XmlWriter::new(&mut file).expect("Error: can't create AST serializer");

        match ast.serialize(&mut writer) {
            Ok(_) => writer.close(),
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }

    fn serialize_symbol_table(&self, symbol_table: &SymbolTable) {
        let mut stream = std::fs::File::create(format!("{}.symbol_table.txt", &self.name))
            .expect("Error: can't create file for serializing symbol table");

        if let Err(err) = symbol_table.traverse(&mut stream) {
            eprintln!("Error: {}", err);
        }
    }
}

#[derive(Default)]
pub struct UnitBuilder {
    unit: Unit,
}

impl UnitBuilder {
    pub fn build(self) -> Unit {
        self.unit
    }

    pub fn set_name(mut self, name: String) -> Self {
        self.unit.name = name;
        self
    }

    pub fn set_path(mut self, path: String) -> Self {
        self.unit.path = path;
        self
    }
}

lazy_static! {
    pub static ref COMPILE_OPTIONS: Mutex<CompileOptions> = Mutex::new(CompileOptions::default());
    pub static ref UNITS: Mutex<Vec<Unit>> = Mutex::new(vec![]);
}

pub fn set_compile_options(opt: CompileOptions) {
    *COMPILE_OPTIONS.lock().unwrap() = opt;
}

pub fn get_compile_options() -> CompileOptions {
    *COMPILE_OPTIONS.lock().unwrap()
}

pub fn register_unit(unit: Unit) {
    UNITS.lock().unwrap().push(unit);
}
