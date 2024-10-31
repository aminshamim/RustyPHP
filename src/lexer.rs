#[derive(Debug, PartialEq, Clone)]
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
    EOF,
}

pub fn lex(input: &str) -> Vec<Token> {
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
            'e' if chars.clone().collect::<String>().starts_with("echo") => {
                chars.by_ref().take(4).for_each(drop); // Move forward
                tokens.push(Token::Echo);
            }
            'p' => {
                let keyword: String = chars.by_ref().take(5).collect();
                if keyword == "print" {
                    tokens.push(Token::Print);
                }
            }
            'i' => {
                let keyword: String = chars.by_ref().take(2).collect();
                if keyword == "if" {
                    tokens.push(Token::If);
                }
            }
            'e' => {
                let keyword: String = chars.by_ref().take(4).collect();
                if keyword == "else" {
                    tokens.push(Token::Else);
                }
            }
            'w' => {
                let keyword: String = chars.by_ref().take(5).collect();
                if keyword == "while" {
                    tokens.push(Token::While);
                }
            }
            'f' => {
                let keyword: String = chars.by_ref().take(3).collect();
                if keyword == "for" {
                    tokens.push(Token::For);
                }
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
            _ => {
                chars.next();
            }
        }
    }

    tokens.push(Token::EOF);
    tokens
}
