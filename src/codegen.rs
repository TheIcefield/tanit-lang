#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CodeGenMode {
    Unset,
    Both,
    HeaderOnly,
    SourceOnly,
}

pub struct CodeGenStream {
    header: std::fs::File,
    source: std::fs::File,
    pub mode: CodeGenMode,
}

impl CodeGenStream {
    pub fn new(name: &str) -> std::io::Result<Self> {
        Ok(Self {
            header: std::fs::File::create(format!("{}_generated.h", name))?,
            source: std::fs::File::create(format!("{}_generated.c", name))?,
            mode: CodeGenMode::Unset,
        })
    }
}

impl std::io::Write for CodeGenStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut res = 0;

        if CodeGenMode::HeaderOnly == self.mode || CodeGenMode::Both == self.mode {
            res += self.header.write(buf)?;
        }

        if CodeGenMode::SourceOnly == self.mode || CodeGenMode::Both == self.mode {
            res += self.source.write(buf)?;
        }

        Ok(res)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        if CodeGenMode::HeaderOnly == self.mode || CodeGenMode::Both == self.mode {
            self.header.write_fmt(fmt)?;
        }

        if CodeGenMode::SourceOnly == self.mode || CodeGenMode::Both == self.mode {
            self.source.write_fmt(fmt)?;
        }

        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.header.flush()?;
        self.source.flush()?;
        Ok(())
    }
}
