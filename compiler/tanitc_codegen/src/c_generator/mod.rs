pub mod ast;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CodeGenMode {
    Unset,
    Both,
    HeaderOnly,
    SourceOnly,
}

pub struct CodeGenStream<'a> {
    header_stream: &'a mut dyn std::io::Write,
    source_stream: &'a mut dyn std::io::Write,
    pub mode: CodeGenMode,
}

impl<'a> CodeGenStream<'a> {
    pub fn new(
        header_stream: &'a mut dyn std::io::Write,
        source_stream: &'a mut dyn std::io::Write,
    ) -> std::io::Result<Self> {
        Ok(Self {
            header_stream,
            source_stream,
            mode: CodeGenMode::Unset,
        })
    }
}

impl std::io::Write for CodeGenStream<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut res = 0;

        if CodeGenMode::HeaderOnly == self.mode || CodeGenMode::Both == self.mode {
            res += self.header_stream.write(buf)?;
        }

        if CodeGenMode::SourceOnly == self.mode || CodeGenMode::Both == self.mode {
            res += self.source_stream.write(buf)?;
        }

        Ok(res)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        if CodeGenMode::HeaderOnly == self.mode || CodeGenMode::Both == self.mode {
            self.header_stream.write_fmt(fmt)?;
        }

        if CodeGenMode::SourceOnly == self.mode || CodeGenMode::Both == self.mode {
            self.source_stream.write_fmt(fmt)?;
        }

        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.header_stream.flush()?;
        self.source_stream.flush()?;
        Ok(())
    }
}
