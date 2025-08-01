pub mod ast;

pub struct RonWriter<'a> {
    stream: &'a mut dyn std::io::Write,
}

impl<'a> RonWriter<'a> {
    pub fn new(stream: &'a mut dyn std::io::Write) -> Result<Self, &'static str> {
        Ok(Self { stream })
    }
}
