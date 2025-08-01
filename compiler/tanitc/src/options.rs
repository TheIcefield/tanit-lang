use std::path::PathBuf;

use tanitc_options::{Backend, CompileOptions, CrateType, SerializationOption};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Argument {
    Short(String),
    Long(String),
    Unknown(String),
}

impl From<String> for Argument {
    fn from(value: String) -> Self {
        if value.starts_with("--") {
            return Self::Long(value);
        }

        if value.starts_with("-") {
            return Self::Short(value);
        }

        Self::Unknown(value)
    }
}

pub struct CommandLineParser {
    args: Vec<String>,
    options: CompileOptions,
    offset: usize,
}

// Public methods
impl CommandLineParser {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            options: CompileOptions::default(),
            offset: 1,
        }
    }

    pub fn parse(&mut self) -> Result<CompileOptions, String> {
        loop {
            let next = self.current_argument();
            if next.is_none() {
                break;
            }
            let next = next.unwrap();

            match next {
                Argument::Short(_) => self.parse_short_option()?,
                Argument::Long(_) => self.parse_long_option()?,
                Argument::Unknown(opt) => return Err(format!("Unexpected option \"{opt}\"")),
            }
        }

        Ok(self.options.clone())
    }

    fn current_token(&self) -> Option<String> {
        if self.args.len() > self.offset {
            Some(self.args[self.offset].to_string())
        } else {
            None
        }
    }

    fn next_token(&mut self) -> Option<String> {
        let tkn = self.current_token();
        self.offset += 1;
        tkn
    }

    fn current_argument(&mut self) -> Option<Argument> {
        Some(Argument::from(self.current_token()?))
    }

    fn next_argument(&mut self) -> Option<Argument> {
        Some(Argument::from(self.next_token()?))
    }
}

impl CommandLineParser {
    fn parse_short_option(&mut self) -> Result<(), String> {
        let Some(next) = self.next_argument() else {
            return Err("Unexpected end of options".to_string());
        };

        let Argument::Short(option) = next else {
            return Err(format!("Expected short option, actually: {next:?}"));
        };

        if let Some(stripped) = option.strip_prefix("-l") {
            self.options.libraries.push(stripped.to_string());
            return Ok(());
        }

        match &option[..] {
            "-L" => {
                let Some(path) = self.next_token() else {
                    return Err("Library path is not set".to_string());
                };

                self.options.libraries_paths.push(PathBuf::from(path));

                Ok(())
            }
            "-o" => {
                let Some(output) = self.next_token() else {
                    return Err("Library path is not set".to_string());
                };

                self.options.output_file = output;

                Ok(())
            }
            "-i" => {
                let Some(input) = self.next_token() else {
                    return Err("Library path is not set".to_string());
                };

                self.options.input_file = input;

                Ok(())
            }
            _ => Err(format!("Unexpected short option: {option}")),
        }
    }

    fn parse_long_option(&mut self) -> Result<(), String> {
        let Some(next) = self.next_argument() else {
            return Err("Unexpected end of options".to_string());
        };

        let Argument::Long(option) = next else {
            return Err(format!("Expected long option, actually: {next:?}"));
        };

        match &option[..] {
            "--dump-tokens" => {
                self.options.verbose_tokens = true;
                Ok(())
            }
            "--dump-ast" => {
                self.options.dump_ast_mode = SerializationOption::Enabled;
                Ok(())
            }
            "--variants" => {
                self.options.allow_variants = true;
                Ok(())
            }
            "--crate-type" => self.parse_crate_type(),
            "--crate-name" => self.parse_crate_name(),
            "--backend" => self.parse_backend(),
            _ => Err(format!("Unexpected short option: {option}")),
        }
    }
}

impl CommandLineParser {
    fn parse_crate_type(&mut self) -> Result<(), String> {
        let Some(next) = self.next_token() else {
            return Err("Crate type is not set".to_string());
        };

        match &next[..] {
            "bin" => self.options.crate_type = CrateType::Bin,
            "static-lib" => self.options.crate_type = CrateType::StaticLib,
            "dynamic-lib" => self.options.crate_type = CrateType::DynamicLib,
            _ => {
                return Err(format!("Unknown crate type: {next}"));
            }
        }

        Ok(())
    }

    fn parse_crate_name(&mut self) -> Result<(), String> {
        let Some(next) = self.next_token() else {
            return Err("Crate name is not set".to_string());
        };

        self.options.crate_name = next;

        Ok(())
    }

    fn parse_backend(&mut self) -> Result<(), String> {
        let Some(next) = self.next_token() else {
            return Err("Backend is not set".to_string());
        };

        match &next[..] {
            "gcc" => self.options.backend = Backend::Gcc,
            "clang" => self.options.backend = Backend::Clang,
            _ => return Err(format!("Unknown backend: {next}")),
        }
        Ok(())
    }
}

#[test]
fn parser_token_test() {
    let args = vec![
        "tanitc".to_string(),
        "-i".to_string(),
        "examples/colors/mod.tt".to_string(),
        "--crate-type".to_string(),
        "static-lib".to_string(),
    ];

    let mut parser = CommandLineParser::new(args);

    assert_eq!(parser.current_token().unwrap(), "-i");
    assert_eq!(parser.next_token().unwrap(), "-i");
    assert_eq!(parser.current_token().unwrap(), "examples/colors/mod.tt");
    assert_eq!(parser.next_token().unwrap(), "examples/colors/mod.tt");
    assert_eq!(parser.current_token().unwrap(), "--crate-type");
    assert_eq!(parser.next_token().unwrap(), "--crate-type");
    assert_eq!(parser.current_token().unwrap(), "static-lib");
    assert_eq!(parser.next_token().unwrap(), "static-lib");
}

#[test]
fn parser_argument_test() {
    let args = vec![
        "tanitc".to_string(),
        "-i".to_string(),
        "examples/colors/mod.tt".to_string(),
        "--crate-type".to_string(),
        "static-lib".to_string(),
    ];

    let mut parser = CommandLineParser::new(args);

    assert_eq!(
        parser.current_argument().unwrap(),
        Argument::Short("-i".to_string())
    );
    assert_eq!(
        parser.next_argument().unwrap(),
        Argument::Short("-i".to_string())
    );
    assert_eq!(parser.current_token().unwrap(), "examples/colors/mod.tt");
    assert_eq!(parser.next_token().unwrap(), "examples/colors/mod.tt");
    assert_eq!(
        parser.current_argument().unwrap(),
        Argument::Long("--crate-type".to_string())
    );
    assert_eq!(
        parser.next_argument().unwrap(),
        Argument::Long("--crate-type".to_string())
    );
    assert_eq!(
        parser.current_argument().unwrap(),
        Argument::Unknown("static-lib".to_string())
    );
    assert_eq!(parser.current_token().unwrap(), "static-lib");
    assert_eq!(parser.next_token().unwrap(), "static-lib");
}

#[test]
fn parser_dump_ast_test() {
    let args = vec!["tanitc".to_string(), "--dump-ast".to_string()];

    let mut parser = CommandLineParser::new(args);

    let options = parser.parse().unwrap();
    assert_eq!(options.dump_ast_mode, SerializationOption::Enabled);
}

#[test]
fn parser_dump_tokens_test() {
    let args = vec!["tanitc".to_string(), "--dump-tokens".to_string()];

    let mut parser = CommandLineParser::new(args);

    let options = parser.parse().unwrap();

    assert_eq!(options.verbose_tokens, true);
}

#[test]
fn parser_crate_type_test() {
    let args = vec![
        "tanitc".to_string(),
        "--crate-type".to_string(),
        "bin".to_string(),
    ];

    let mut parser = CommandLineParser::new(args);

    let options = parser.parse().unwrap();
    assert_eq!(options.crate_type, CrateType::Bin);
}

#[test]
fn parser_crate_name_test() {
    let args = vec![
        "tanitc".to_string(),
        "--crate-name".to_string(),
        "hello".to_string(),
    ];

    let mut parser = CommandLineParser::new(args);

    let options = parser.parse().unwrap();
    assert_eq!(options.crate_name, "hello");
}

#[test]
fn parser_input_output_files_test() {
    let args = vec![
        "tanitc".to_string(),
        "-i".to_string(),
        "hello.tt".to_string(),
        "-o".to_string(),
        "hello".to_string(),
    ];

    let mut parser = CommandLineParser::new(args);

    let options = parser.parse().unwrap();
    assert_eq!(options.input_file, "hello.tt");
    assert_eq!(options.output_file, "hello");
}
