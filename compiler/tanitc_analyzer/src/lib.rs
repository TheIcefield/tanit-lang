use tanitc_ast::attributes::Safety;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::{Errors, Message, Warnings};
use tanitc_options::CompileOptions;
use tanitc_symbol_table::{entry::Entry, table::Table};
use tanitc_ty::Type;

pub mod ast;

pub type Counter = usize;

pub trait Analyze {
    fn get_type(&self, _analyzer: &mut Analyzer) -> Type {
        Type::unit()
    }

    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message>;
}

pub struct Analyzer {
    pub table: Box<Table>,
    compile_options: CompileOptions,
    counter: Counter,
    errors: Errors,
    warnings: Warnings,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            table: Box::new(Table::new()),
            counter: 0,
            errors: Errors::new(),
            warnings: Warnings::new(),
            compile_options: CompileOptions::default(),
        }
    }

    pub fn with_options(compile_options: CompileOptions) -> Self {
        Self {
            table: Box::new(Table::new()),
            counter: 0,
            errors: Errors::new(),
            warnings: Warnings::new(),
            compile_options,
        }
    }

    pub fn counter(&mut self) -> Counter {
        let old = self.counter;
        self.counter += 1;
        old
    }

    pub fn get_current_safety(&self) -> Safety {
        self.table.get_safety()
    }

    pub fn get_table(&self) -> &Table {
        &self.table
    }

    pub fn has_symbol(&self, name: Ident) -> bool {
        self.table.lookup(name).is_some()
    }

    pub fn add_symbol(&mut self, entry: Entry) {
        self.table.insert(entry);
    }

    pub fn check_main(&self) -> Result<(), Message> {
        if self.table.lookup(Ident::from("main".to_string())).is_none() {
            return Err(Message::new(Location::new(), "No entry point!"));
        }

        Ok(())
    }

    pub fn error(&mut self, mut error: Message) {
        error.text = format!("Semantic error: {}", error.text);
        self.errors.push(error);
    }

    pub fn warning(&mut self, mut warn: Message) {
        warn.text = format!("Semantic warning: {}", warn.text);
        self.warnings.push(warn);
    }

    pub fn get_errors(&mut self) -> Errors {
        std::mem::take(&mut self.errors)
    }

    pub fn get_warnings(&mut self) -> Warnings {
        std::mem::take(&mut self.warnings)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}
