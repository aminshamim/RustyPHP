// lexer.rs

use serde::Serialize;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Token {
    PhpOpen,
    PhpClose,
    Echo,
    Print,
    If,
    Else,
    While,
    For,
    Variable(String),
    Equals,
    Number(f64),
    String(String),
    Semicolon,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Plus,
    Minus,
    Multiply,
    Divide,
    Dot, // Token for concatenation
    EOF,
}

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '<' => {
                let next: String = chars.by_ref().take(5).collect();
                if next == "<?php" {
                    tokens.push(Token::PhpOpen);
                }
            }
            '?' => {
                let next: String = chars.by_ref().take(2).collect();
                if next == "?>" {
                    tokens.push(Token::PhpClose);
                }
            }
            'e' => {
                let next: String = chars.clone().take(4).collect();
                if next == "echo" {
                    chars.by_ref().take(4).for_each(drop);
                    tokens.push(Token::Echo);
                } else if next == "else" {
                    chars.by_ref().take(4).for_each(drop);
                    tokens.push(Token::Else);
                } else {
                    return Err("Unexpected token: expected 'echo' or 'else'".to_string());
                }
            }
            'p' => {
                let next: String = chars.clone().take(5).collect();
                if next == "print" {
                    chars.by_ref().take(5).for_each(drop);
                    tokens.push(Token::Print);
                } else if next == "else" {
                    chars.by_ref().take(5).for_each(drop);
                    tokens.push(Token::Else);
                } else {
                    return Err("Unexpected token: expected 'echo' or 'else'".to_string());
                }
            }
            '.' => {
                tokens.push(Token::Dot); // Add support for Dot token
                chars.next();
            }
            '=' => {
                tokens.push(Token::Equals);
                chars.next();
            }
            ';' => {
                tokens.push(Token::Semicolon);
                chars.next();
            }
            '(' => {
                tokens.push(Token::OpenParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::CloseParen);
                chars.next();
            }
            '{' => {
                tokens.push(Token::OpenBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::CloseBrace);
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Multiply);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Divide);
                chars.next();
            }
            '"' | '\'' => {
                let quote = chars.next().unwrap();
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == quote {
                        chars.next();
                        break;
                    }
                    s.push(ch);
                    chars.next();
                }
                tokens.push(Token::String(s));
            }
            '0'..='9' => {
                let mut num_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_numeric() || ch == '.' {
                        num_str.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Ok(number) = num_str.parse::<f64>() {
                    tokens.push(Token::Number(number));
                }
            }
            _ if c.is_whitespace() => {
                chars.next();
            }
            '$' => {
                chars.next(); // Skip '$'
                let mut var_name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        var_name.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Variable(var_name));
            }
            _ => {
                chars.next();
            }
        }
    }

    tokens.push(Token::EOF);
    Ok(tokens)
}
