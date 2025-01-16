use tanit::{
    analyzer::{self, symbol_table::SymbolTable},
    ast, codegen,
    messages::{Error, Warning},
    parser, serializer,
};

fn serialize_ast(output: &str, ast: &ast::Ast) {
    let mut file = std::fs::File::create(format!("{}_ast.xml", output))
        .expect("Error: can't create file for dumping AST");

    let mut writer =
        serializer::XmlWriter::new(&mut file).expect("Error: can't create AST serializer");

    match ast.serialize(&mut writer) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}

fn serialize_symbol_table(output: &str, symbol_table: &SymbolTable) {
    let mut stream = std::fs::File::create(format!("{}_symbol_table.txt", output))
        .expect("Error: can't create file for serializing symbol table");

    if let Err(err) = symbol_table.traverse(&mut stream) {
        eprintln!("Error: {}", err);
    }
}

fn generate_code(output: &str, ast: &ast::Ast) {
    let mut header_stream = std::fs::File::create(format!("{}_generated.h", output))
        .expect("Error: can't create file for header stream");
    let mut source_stream = std::fs::File::create(format!("{}_generated.h", output))
        .expect("Error: can't create file for header stream");

    let mut writer = codegen::CodeGenStream::new(&mut header_stream, &mut source_stream)
        .expect("Error: can't create codegen writer");

    if let Err(err) = ast.codegen(&mut writer) {
        eprintln!("Error: {}", err);
    }
}

fn print_errors(errors: &[Error]) {
    for err in errors.iter() {
        eprintln!("{}: {}", err.location, err.text);
    }
}

fn print_warnings(warnings: &[Warning]) {
    for warn in warnings.iter() {
        eprintln!("{}: {}", warn.location, warn.text);
    }
}

fn print_messages(errors: &[Error], warnings: &[Warning]) {
    print_errors(errors);
    print_warnings(warnings);
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

    let mut parser = parser::Parser::new(
        parser::lexer::Lexer::from_file(&source_file, dump_tokens)
            .expect("Error: can't create lexer"),
    );

    let mut ast = match parser.parse() {
        Ok(ast) => ast,
        Err(messages) => {
            print_messages(&messages.0, &messages.1);
            return;
        }
    };

    let mut analyzer = analyzer::Analyzer::new();

    let (symtable, errors, warnings) = analyzer.analyze(&mut ast);

    if dump_ast {
        serialize_ast(&output_file, &ast);
    }

    if dump_symtable {
        serialize_symbol_table(&output_file, &symtable);
    }

    if !errors.is_empty() {
        print_errors(&errors);
        return;
    }

    print_warnings(&warnings);

    generate_code(&output_file, &ast);

    println!("C code generated")
}
