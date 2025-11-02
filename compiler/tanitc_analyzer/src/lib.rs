use tanitc_attributes::Safety;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::{listener::MessageListener, Message};
use tanitc_options::CompileOptions;
use tanitc_symbol_table::{
    entry::{Entry, SymbolKind},
    table::Table,
};
use tanitc_ty::Type;

pub mod ast;

pub type Counter = usize;

pub struct Analyzer {
    pub table: Box<Table>,
    compile_options: CompileOptions,
    counter: Counter,
    messages: MessageListener,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            table: Box::new(Table::new()),
            counter: 0,
            messages: MessageListener::new(),
            compile_options: CompileOptions::default(),
        }
    }

    pub fn set_compile_options(&mut self, compile_options: CompileOptions) {
        self.compile_options = compile_options;
    }

    pub fn set_message_listener(&mut self, messages: MessageListener) {
        self.messages = messages;
    }

    pub fn messages_ref(&self) -> &MessageListener {
        &self.messages
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

    pub fn check_entry_point(&self) -> Result<(), Message> {
        const ENTRY_POINT: &str = "main";
        let main_func_id = Ident::from(ENTRY_POINT.to_string());

        let Some(entry) = self.table.lookup(main_func_id) else {
            return Err(Message::new(&Location::default(), "No entry point!"));
        };

        let SymbolKind::FuncDef(data) = &entry.kind else {
            return Err(Message::new(
                &Location::default(),
                "No entry point function!",
            ));
        };

        if data.return_type != Type::I32 && !data.return_type.is_unit() {
            return Err(Message::from_string(
                &Location::default(),
                format!("Bad type of main function: {}", data.return_type),
            ));
        }

        Ok(())
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

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}
