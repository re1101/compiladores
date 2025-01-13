mod lexer;
use lexer::Lexer;

mod parser;
use parser::Parser;

mod token;

mod semantic_analyzer;

mod compiler_error;

use std::env;
use std::fs::File;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]); // Se pasa el nombre del archivo ejecutable como argumento
        std::process::exit(1);
    }

    let filename = &args[1];
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut lexer = Lexer::new(&contents);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    println!("AST: {:?}", ast);

    if let Some(ast) = ast {
        let mut semantic_analyzer = semantic_analyzer::SemanticAnalyzer::new();
        match semantic_analyzer.analyze(&ast) {
            Ok(_) => println!("Semantic analysis passed"),
            Err(e) => println!("Semantic analysis error: {}", e),
        }
    } else {
        println!("Failed to parse input");
    }

    Ok(())
}