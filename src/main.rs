use garnet_script::{lexer, error_listener, parser};

fn main() {
    let mut source_file = "main.grs".to_string();
    let mut output_file = "a".to_string();
    let mut dump_tokens = false;
    let mut dump_ast = true;

    let argv = std::env::args().collect::<Vec<String>>();
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

    let ast = parser.parse();

    let ast = match ast {
        Err(errors) => {
            errors.dump_errors();
            return;
        }

        Ok(ast) => ast,
    };

    if dump_ast {
        match parser::dump_ast(output_file, &ast) {
            Err(err) => println!("{}", err),
            _ => {}
        }
    }
    
}
