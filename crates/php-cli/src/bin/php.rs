//! PHP CLI Binary
//! 
//! This is the main CLI binary for RustyPHP

use std::env;
use std::fs;
use std::process;
use php_lexer;
use php_parser::ast::Stmt;
use php_runtime::Engine;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <php_file>", args[0]);
        process::exit(1);
    }
    
    let filename = &args[1];
    
    // Read the PHP file
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };
    
    // Tokenize
    let tokens = match php_lexer::lex(&content) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            process::exit(1);
        }
    };
    
    // Parse
    let ast = match php_parser::parse(tokens) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parser error: {}", e);
            process::exit(1);
        }
    };
    
    // Execute
    let mut engine = Engine::new();
    if let Err(e) = engine.execute_stmt(&ast) {
        eprintln!("Runtime error: {}", e);
        process::exit(1);
    }
    
    // Print output
    print!("{}", engine.get_output());
}
