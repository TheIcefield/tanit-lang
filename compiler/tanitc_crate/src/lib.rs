use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use tanitc_analyzer::{self, Analyzer};
use tanitc_ast::Ast;
use tanitc_builder::{build_object_file, link_crate_objects};
use tanitc_codegen::c_generator::{CodeGenMode, CodeGenStream};
use tanitc_lexer::Lexer;
use tanitc_options::{AstSerializeMode, CompileOptions, CrateType};
use tanitc_parser::Parser;
use tanitc_symbol_table::table::Table;

use lazy_static::lazy_static;

pub mod ast;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum UnitProcessState {
    #[default]
    NotProcessed,
    Parsed,
    Analyzed,
    Generated,
    Processed,
    Failed,
}

#[derive(Clone)]
pub struct Unit {
    name: String,
    path: String,
    generated_path: String,
    built_path: String,
    process_state: UnitProcessState,
    ast: Option<Ast>,
    symbol_table: Option<Table>,
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            name: "main".to_string(),
            path: "./main.tt".to_string(),
            generated_path: String::default(),
            built_path: String::default(),
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

    fn parse_program(parser: &mut Parser) -> Option<Ast> {
        let res = parser.parse_global_block();

        if let Err(err) = &res {
            parser.error(err.clone());
        }

        if parser.has_errors() {
            None
        } else {
            Some(res.unwrap())
        }
    }

    pub fn find_modules(&mut self) -> Vec<Unit> {
        let mut searcher = ast::ModuleSearcher {
            current_path: self.path.clone(),
            subunits: vec![],
        };

        if let Err(e) = self.ast.as_mut().unwrap().accept(&mut searcher) {
            eprintln!("{e}");
        }

        std::mem::take(&mut searcher.subunits)
    }

    pub fn process_parsing(&mut self) -> Result<(), &'static str> {
        if !matches!(self.process_state, UnitProcessState::NotProcessed) {
            return Err("Error: expected \"NotProcessed\" state");
        }

        print!("Parsing: \"{}\"... ", &self.path);

        let mut lexer = Lexer::from_file(&self.path)?;
        lexer.verbose_tokens = get_compile_options().verbose_tokens;

        let mut parser = Parser::new(lexer);

        self.ast = Self::parse_program(&mut parser);

        if parser.has_errors() || self.ast.is_none() {
            println!("FAIL!");
            tanitc_messages::print_messages(&parser.get_errors());
            self.process_state = UnitProcessState::Failed;
            return Err("Parse errors occured");
        }

        if parser.has_warnings() {
            tanitc_messages::print_messages(&parser.get_warnings());
        }

        self.process_state = UnitProcessState::Parsed;

        println!("OK!");

        Ok(())
    }

    fn analyze_program(ast: &mut Ast, analyzer: &mut Analyzer) -> Option<Table> {
        if let Err(err) = ast.accept_mut(analyzer) {
            analyzer.error(err);
        }

        let compile_options = get_compile_options();
        if compile_options.crate_type == CrateType::Bin {
            if let Err(err) = analyzer.check_main() {
                analyzer.error(err);
            }
        }

        if analyzer.has_errors() {
            None
        } else {
            Some(std::mem::take(&mut analyzer.table))
        }
    }

    pub fn process_analyze(&mut self) -> Result<(), &'static str> {
        if !matches!(self.process_state, UnitProcessState::Parsed) {
            return Err("Error: expected \"Parsed\" stated");
        }

        if self.ast.is_none() {
            return Err("Error: required \"AST\"");
        }

        print!("Analyzing: \"{}\"... ", &self.path);

        let mut analyzer = Analyzer::with_options(get_compile_options());
        self.symbol_table = Self::analyze_program(self.ast.as_mut().unwrap(), &mut analyzer);

        if analyzer.has_errors() || self.symbol_table.is_none() {
            println!("FAIL!");
            tanitc_messages::print_messages(&analyzer.get_errors());
            self.process_state = UnitProcessState::Failed;
            return Err("Analyze errors occured");
        }

        if analyzer.has_warnings() {
            tanitc_messages::print_messages(&analyzer.get_warnings());
        }

        self.process_state = UnitProcessState::Analyzed;

        println!("OK!");

        let compile_options = get_compile_options();

        if AstSerializeMode::None != compile_options.dump_ast_mode {
            print!("Serializing AST: \"{}\"... ", &self.path);
            self.serialize_ast(compile_options.dump_ast_mode);
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
            return Err("Error: expected \"Analyzed\" stated");
        }

        if self.ast.is_none() {
            return Err("Error: missing required \"AST\"");
        }

        if self.symbol_table.is_none() {
            return Err("Error: missing required \"Symbol table\"");
        }

        print!("Generating \"{}\"... ", &self.path);

        self.generated_path = format!("{}.tt.c", &self.name);

        let mut header_stream = std::fs::File::create(format!("{}.tt.h", &self.name))
            .expect("Error: can't create file for header stream");
        let mut source_stream = std::fs::File::create(&self.generated_path)
            .expect("Error: can't create file for source stream");

        let mut writer = CodeGenStream::new(&mut header_stream, &mut source_stream)
            .expect("Error: can't create codegen writer");

        let old_mode = writer.mode;
        writer.mode = CodeGenMode::SourceOnly;

        writeln!(writer, "#include \"{}.tt.h\"\n", &self.name).unwrap();

        writer.mode = old_mode;

        if let Err(err) = self.ast.as_mut().unwrap().accept(&mut writer) {
            self.process_state = UnitProcessState::Failed;
            eprintln!("Error: {err}");
        } else {
            self.process_state = UnitProcessState::Generated;
            println!("OK!");
        }

        Ok(())
    }

    pub fn process_building(&mut self) -> Result<(), &'static str> {
        if !matches!(self.process_state, UnitProcessState::Generated) {
            return Err("Error: expected \"Generated\" stated");
        }

        let compile_options = get_compile_options();

        self.built_path = format!("{}.o", &self.name);

        print!("Building AST: \"{}\"... ", &self.path);

        match build_object_file(
            Path::new(&self.generated_path),
            Path::new(&self.built_path),
            &compile_options,
        ) {
            Ok(_) => {
                self.process_state = UnitProcessState::Processed;
                println!("OK!");
            }
            Err(err) => {
                self.process_state = UnitProcessState::Failed;
                eprintln!("{err}");
            }
        }

        Ok(())
    }

    pub fn process() -> Result<(), &'static str> {
        loop {
            let mut failed = false;
            let mut is_all_processed = true;
            let mut sub_units: Vec<Unit> = Vec::new();

            for unit in UNITS.lock().unwrap().iter_mut() {
                match unit.process_state {
                    UnitProcessState::NotProcessed => {
                        is_all_processed = false;
                        if let Err(e) = unit.process_parsing() {
                            eprintln!("{e}");
                        }
                    }
                    UnitProcessState::Parsed => {
                        is_all_processed = false;
                        sub_units = unit.find_modules();
                        if let Err(e) = unit.process_analyze() {
                            eprintln!("{e}");
                        }
                    }
                    UnitProcessState::Analyzed => {
                        is_all_processed = false;
                        if let Err(e) = unit.process_codegen() {
                            eprintln!("{e}");
                        }
                    }
                    UnitProcessState::Generated => {
                        if let Err(e) = unit.process_building() {
                            eprintln!("{e}");
                        }
                    }
                    UnitProcessState::Failed => {
                        failed = true;
                    }

                    _ => {}
                }
            }

            for sub_unit in sub_units.iter() {
                register_unit(sub_unit.clone());
            }

            if is_all_processed || failed {
                break;
            }
        }

        let mut sources = Vec::<PathBuf>::new();
        for unit in UNITS.lock().unwrap().iter_mut() {
            sources.push(unit.built_path.clone().into());
        }

        let compile_options = get_compile_options();
        if let Err(err) = link_crate_objects(&sources, &compile_options) {
            eprintln!("{err}");
        }

        Ok(())
    }

    fn serialize_ron(&mut self, file: &mut dyn std::io::Write) {
        use tanitc_serializer::ron_writer::RonWriter;
        let mut writer = RonWriter::new(file).expect("Error: can't create AST serializer");

        match self.ast.as_mut().unwrap().accept(&mut writer) {
            Ok(_) => {}
            Err(err) => eprintln!("Error: {err}"),
        }
    }

    fn serialize_xml(&mut self, file: &mut dyn std::io::Write) {
        use tanitc_serializer::xml_writer::XmlWriter;
        let mut writer = XmlWriter::new(file).expect("Error: can't create AST serializer");

        match self.ast.as_mut().unwrap().accept(&mut writer) {
            Ok(_) => writer.close(),
            Err(err) => eprintln!("Error: {err}"),
        }
    }

    fn serialize_json(&mut self, _file: &mut dyn std::io::Write) {
        eprintln!("JSON serialization is not yet supported");
    }

    fn serialize_ast(&mut self, mode: AstSerializeMode) {
        let suffix = match mode {
            AstSerializeMode::None => "",
            AstSerializeMode::Ron => "ron",
            AstSerializeMode::Xml => "xml",
            AstSerializeMode::Json => "json",
        };

        let mut file = std::fs::File::create(format!("{}.ast.{suffix}", &self.name))
            .expect("Error: can't create file for dumping AST");

        match mode {
            AstSerializeMode::None => {}
            AstSerializeMode::Ron => self.serialize_ron(&mut file),
            AstSerializeMode::Xml => self.serialize_xml(&mut file),
            AstSerializeMode::Json => self.serialize_json(&mut file),
        }
    }

    fn serialize_symbol_table(&self, symbol_table: &Table) {
        use std::io::Write;

        let mut stream = std::fs::File::create(format!("{}.symbol_table.txt", &self.name))
            .expect("Error: can't create file for serializing symbol table");

        if let Err(err) = write!(stream, "{symbol_table:#?}") {
            eprintln!("Error: {err}");
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
    COMPILE_OPTIONS.lock().unwrap().clone()
}

pub fn register_unit(unit: Unit) {
    UNITS.lock().unwrap().push(unit);
}
