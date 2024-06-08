use tanit::{analyzer, error_listener, lexer, parser};

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
        let lexer = lexer::Lexer::from_file(&source_file, dump_tokens);
        match lexer {
            Err(err) => {
                println!("Error when open file \"{}\": {}", source_file, err);
                return;
            }
            Ok(lexer) => lexer,
        }
    };

    let error_listener = error_listener::ErrorListener::new();

    let mut parser = parser::Parser::new(lexer, error_listener);
    let mut ast = match parser.parse() {
        Ok(ast) => ast,
        Err(error_listener) => {
            error_listener.dump_errors();
            return;
        }
    };

    let error_listener = parser.error_listener();
    let mut analyzer = analyzer::Analyzer::new(error_listener);

    let (symtable, errors) = analyzer.analyze(&mut ast);

    if dump_ast {
        if let Err(err) = parser::dump_ast(output_file.clone(), &ast) {
            println!("{}", err);
        }
    }

    if !errors.is_empty() {
        errors.dump_errors();
    }

    if dump_symtable {
        let _ = analyzer::dump_symtable(output_file, &symtable);
    }
}
