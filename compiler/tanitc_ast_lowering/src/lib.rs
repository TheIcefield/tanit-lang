use tanitc_ast::program_ctx::ProgramCtx;
use tanitc_hir::hir::Hir;
use tanitc_messages::{listener::MessageListener, Message};

pub(crate) mod program_ctx;

pub struct AstLowering<'ast> {
    ast: &'ast ProgramCtx,
    messages: MessageListener,
}

pub type AstLowResult<T> = Result<T, Message>;

impl<'ast> AstLowering<'ast> {
    pub fn new(ast: &'ast ProgramCtx) -> Self {
        Self {
            ast,
            messages: MessageListener::new(),
        }
    }

    pub fn low(&mut self) -> Result<Box<Hir>, MessageListener> {
        match self.low_program_ctx(self.ast) {
            Ok(hir) => Ok(Box::new(hir)),
            Err(msg) => {
                self.error(msg);
                Err(std::mem::take(self.messages_mut()))
            }
        }
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
