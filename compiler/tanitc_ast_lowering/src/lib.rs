use tanitc_ast::program_ctx::ProgramCtx;
use tanitc_hir::hir::Hir;
use tanitc_messages::{listener::MessageListener, Message};
use tanitc_options::CompileOptions;

pub(crate) mod program_ctx;

#[derive(Default)]
pub struct AstLowering {
    compile_options: CompileOptions,
    messages: MessageListener,
}

pub type AstLowResult<T> = Result<T, Message>;

impl AstLowering {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_compile_options(compile_options: CompileOptions) -> Self {
        let mut analyzer = Self::new();
        analyzer.set_compile_options(compile_options);
        analyzer
    }

    pub fn low(&mut self, program_ctx: &ProgramCtx) -> Result<Box<Hir>, MessageListener> {
        match self.low_program_ctx(program_ctx) {
            Ok(hir) => Ok(Box::new(hir)),
            Err(msg) => {
                self.error(msg);
                Err(std::mem::take(self.messages_mut()))
            }
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

    pub fn messages_mut(&mut self) -> &mut MessageListener {
        &mut self.messages
    }

    pub fn error(&mut self, mut error: Message) {
        error.text = format!("Syntax error: {}", error.text);
        self.messages.error(error);
    }

    pub fn warning(&mut self, mut warn: Message) {
        warn.text = format!("Syntax warning: {}", warn.text);
        self.messages.warn(warn);
    }
}
