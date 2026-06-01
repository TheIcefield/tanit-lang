use crate::symbol_table::{
    entry::{Entry, SymbolKind},
    table::Table,
};
use tanitc_attributes::Safety;
use tanitc_hir::hir::{type_spec::Type, Hir};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::{listener::MessageListener, Message};
use tanitc_options::{CompileOptions, CrateType};

pub(crate) mod hir;
pub(crate) mod symbol_table;

pub type AnalyzeResult<T> = Result<T, Message>;

pub type Counter = usize;

pub struct Analyzer<'a> {
    pub table: Box<Table>,
    compile_options: &'a CompileOptions,
    counter: Counter,
    messages: MessageListener,
}

impl<'a> Analyzer<'a> {
    pub fn new(compile_options: &'a CompileOptions) -> Self {
        Self {
            table: Box::default(),
            compile_options,
            counter: Counter::default(),
            messages: MessageListener::new(),
        }
    }

    pub fn analyze_program(&mut self, hir: &mut Hir) -> Result<(), MessageListener> {
        if let Err(err) = hir.accept_mut(self) {
            self.error(err);
        }

        if self.compile_options.crate_type == CrateType::Bin {
            if let Err(err) = self.check_entry_point() {
                self.error(err);
            }
        }

        if self.messages_ref().has_errors() {
            return Err(std::mem::take(self.messages_mut()));
        }

        Ok(())
    }

    pub fn set_message_listener(&mut self, messages: MessageListener) {
        self.messages = messages;
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

    pub fn has_symbol(&self, id: Ident) -> bool {
        self.table.lookup(id).is_some()
    }

    pub fn add_symbol(&mut self, entry: Entry) {
        self.table.insert(entry);
    }

    pub fn check_entry_point(&self) -> Result<(), Message> {
        const ENTRY_POINT: &str = "main";
        let main_func_id = Ident::from(ENTRY_POINT.to_string());

        let Some(entry) = self.table.lookup(main_func_id) else {
            return Err(Message::new(Location::default(), "No entry point!"));
        };

        let SymbolKind::FuncDef(data) = &entry.kind else {
            return Err(Message::new(
                Location::default(),
                "No entry point function!",
            ));
        };

        if *data.ty.return_type != Type::I32 && !data.ty.return_type.is_unit() {
            return Err(Message::new(
                Location::default(),
                format!("Bad type of main function: {}", data.ty.return_type),
            ));
        }

        Ok(())
    }

    pub fn messages_ref(&self) -> &MessageListener {
        &self.messages
    }

    pub fn messages_mut(&mut self) -> &mut MessageListener {
        &mut self.messages
    }

    pub fn error(&mut self, mut error: Message) {
        error.text = format!("Semantic error: {}", error.text);
        self.messages.error(error);
    }

    pub fn warning(&mut self, mut warn: Message) {
        warn.text = format!("Semantic warning: {}", warn.text);
        self.messages.warn(warn);
    }
}
