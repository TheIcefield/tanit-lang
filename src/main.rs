use tanit::{
    analyzer, ast, codegen,
    messages::{Error, Warning},
    parser, serializer,
};

fn serialize_ast(output: &str, ast: &ast::Ast) -> Result<(), &'static str> {
    let mut writer = serializer::XmlWriter::new(&format!("{}_ast.xml", output))?;
    match ast.serialize(&mut writer) {
        Ok(_) => Ok(()),
        Err(err) => {
            eprintln!("Error: {}", err);
            Err("Error during serializing AST")
        }
    }
}

fn dump_errors(errors: &[Error]) {
    for err in errors.iter() {
        eprintln!("{}: {}", err.location, err.text);
    }
}

fn dump_warnings(warnings: &[Warning]) {
    for warn in warnings.iter() {
        eprintln!("{}: {}", warn.location, warn.text);
    }
}

fn dump_messages(errors: &[Error], warnings: &[Warning]) {
    dump_errors(errors);
    dump_warnings(warnings);
}

fn main() {
    let mut source_file = "main.tt".to_string();
    let mut output_file = "a".to_string();
    let mut dump_tokens = false;
    let mut dump_ast = true;
    let mut dump_symtable = true;

    let argv = std::env::args().collect::<Vec<String>>();
    #[allow(clippy::needless_range_loop)]
    for mut i in 1..argv.len() {
        if argv[i] == "-i" {
            i += 1;
            source_file = argv[i].clone();
        } else if argv[i] == "-o" {
            i += 1;
            output_file = argv[i].clone();
        } else if argv[i] == "--dump-tokens" {
            dump_tokens = true;
        } else if argv[i] == "--dump-ast" {
            dump_ast = true;
            source_file = argv[i].clone();
        } else if argv[i] == "--dump-symtable" {
            dump_symtable = true;
            source_file = argv[i].clone();
        }
    }

    let lexer = {
        let lexer = parser::lexer::Lexer::from_file(&source_file, dump_tokens);
        match lexer {
            Err(err) => {
                println!("Error when open file \"{}\": {}", source_file, err);
                return;
            }
            Ok(lexer) => lexer,
        }
    };

    let mut parser = parser::Parser::new(lexer);
    let mut ast = match parser.parse() {
        Ok(ast) => ast,
        Err(messages) => {
            dump_messages(&messages.0, &messages.1);
            return;
        }
    };

    let mut analyzer = analyzer::Analyzer::new();

    let (symtable, errors, warnings) = analyzer.analyze(&mut ast);

    if dump_ast {
        if let Err(err) = serialize_ast(&output_file, &ast) {
            println!("{}", err);
        }
    }

    if dump_symtable {
        if let Err(err) = analyzer::dump_symtable(&output_file, &symtable) {
            eprintln!("{}", err);
        }
    }

    if !errors.is_empty() {
        dump_errors(&errors);
        return;
    }

    dump_warnings(&warnings);

    let mut codegen = {
        let codegen = codegen::CodeGenStream::new(&output_file);
        match codegen {
            Ok(codegen) => codegen,
            Err(err) => {
                eprintln!("Error when open file \"{}\": {}", source_file, err);
                return;
            }
        }
    };

    match ast.codegen(&mut codegen) {
        Ok(_) => println!("C code generated"),
        Err(_) => eprintln!("Error occured during C code generating"),
    }
}
