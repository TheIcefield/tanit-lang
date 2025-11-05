use tanitc_hir::hir::Hir;
use tanitc_options::CompileOptions;

pub(crate) mod hir;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum CodeGenMode {
    #[default]
    Unset,
    Both,
    HeaderOnly,
    SourceOnly,
}

pub struct CodeGenStream<'a> {
    header_stream: &'a mut dyn std::io::Write,
    source_stream: &'a mut dyn std::io::Write,
    compile_options: CompileOptions,
    pub mode: CodeGenMode,
    pub indent: usize,
}

impl<'a> CodeGenStream<'a> {
    pub fn new(
        header_stream: &'a mut dyn std::io::Write,
        source_stream: &'a mut dyn std::io::Write,
    ) -> Self {
        Self {
            header_stream,
            source_stream,
            compile_options: CompileOptions::default(),
            mode: CodeGenMode::Unset,
            indent: 0,
        }
    }

    pub fn with_compile_options(
        header_stream: &'a mut dyn std::io::Write,
        source_stream: &'a mut dyn std::io::Write,
        compile_options: CompileOptions,
    ) -> Self {
        Self {
            header_stream,
            source_stream,
            compile_options,
            mode: CodeGenMode::Unset,
            indent: 0,
        }
    }

    pub fn codegen_program(&mut self, program_hir: &Hir) -> Result<(), String> {
        use std::io::Write;

        self.mode = CodeGenMode::SourceOnly;

        let file_name = format!("{}.tt.h", self.compile_options.crate_name);

        writeln!(self, "#include \"{file_name}\"\n").map_err(|err| err.to_string())?;

        self.mode = CodeGenMode::Unset;

        program_hir.accept(self).map_err(|err| err.to_string())?;

        Ok(())
    }

    pub fn indentation(&self) -> String {
        let mut s = String::new();
        for _ in 0..self.indent {
            s.push_str("    ");
        }
        s
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
