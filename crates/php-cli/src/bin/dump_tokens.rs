use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <php_file>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];
    let content = fs::read_to_string(filename).expect("Failed to read file");
    let tokens = php_lexer::lex(&content).expect("Lexing failed");
    for (i, tok) in tokens.iter().enumerate() {
        println!("{:03}: {:?}", i, tok);
    }
}