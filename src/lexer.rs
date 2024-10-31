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
    Function,
    Return,
    Class,
    Extends,
    Implements,
    New,
    Public,
    Private,
    Protected,
    Static,
    Var,
    Const,
    True,
    False,
    Null,
    Isset,
    Empty,
    PrintR,
    Strlen,
    Strpos,
    Substr,
    ArrayPush,
    ArrayPop,
    ArrayMerge,
    InArray,
    Explode,
    Implode,
    Count,
    Switch,
    Case,
    Default,
    Break,
    Continue,
    Do,
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
    Dot,
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
            'a' => {
                let next: String = chars.clone().take(10).collect();
                match &next[..] {
                    "array_push" => {
                        chars.by_ref().take(10).for_each(drop);
                        tokens.push(Token::ArrayPush);
                    }
                    "array_pop" => {
                        chars.by_ref().take(9).for_each(drop);
                        tokens.push(Token::ArrayPop);
                    }
                    "array_merge" => {
                        chars.by_ref().take(11).for_each(drop);
                        tokens.push(Token::ArrayMerge);
                    }
                    _ => {}
                }
            }
            'b' => {
                let next: String = chars.clone().take(5).collect();
                if next == "break" {
                    chars.by_ref().take(5).for_each(drop);
                    tokens.push(Token::Break);
                }
            }
            'c' => {
                let next: String = chars.clone().take(5).collect();
                match &next[..] {
                    "class" => {
                        chars.by_ref().take(5).for_each(drop);
                        tokens.push(Token::Class);
                    }
                    "case" => {
                        chars.by_ref().take(4).for_each(drop);
                        tokens.push(Token::Case);
                    }
                    "const" => {
                        chars.by_ref().take(5).for_each(drop);
                        tokens.push(Token::Const);
                    }
                    "continue" => {
                        chars.by_ref().take(8).for_each(drop);
                        tokens.push(Token::Continue);
                    }
                    "count" => {
                        chars.by_ref().take(5).for_each(drop);
                        tokens.push(Token::Count);
                    }
                    _ => {}
                }
            }
            'd' => {
                let next: String = chars.clone().take(7).collect();
                match &next[..] {
                    "default" => {
                        chars.by_ref().take(7).for_each(drop);
                        tokens.push(Token::Default);
                    }
                    "do" => {
                        chars.by_ref().take(2).for_each(drop);
                        tokens.push(Token::Do);
                    }
                    _ => {}
                }
            }
            'e' => {
                let next: String = chars.clone().take(7).collect();
                match &next[..] {
                    "echo" => {
                        chars.by_ref().take(4).for_each(drop);
                        tokens.push(Token::Echo);
                    }
                    "else" => {
                        chars.by_ref().take(4).for_each(drop);
                        tokens.push(Token::Else);
                    }
                    "empty" => {
                        chars.by_ref().take(5).for_each(drop);
                        tokens.push(Token::Empty);
                    }
                    "extends" => {
                        chars.by_ref().take(7).for_each(drop);
                        tokens.push(Token::Extends);
                    }
                    "explode" => {
                        chars.by_ref().take(7).for_each(drop);
                        tokens.push(Token::Explode);
                    }
                    _ => {}
                }
            }
            'f' => {
                let next: String = chars.clone().take(8).collect();
                match &next[..] {
                    "function" => {
                        chars.by_ref().take(8).for_each(drop);
                        tokens.push(Token::Function);
                    }
                    "for" => {
                        chars.by_ref().take(3).for_each(drop);
                        tokens.push(Token::For);
                    }
                    "false" => {
                        chars.by_ref().take(5).for_each(drop);
                        tokens.push(Token::False);
                    }
                    _ => {}
                }

            }
            'i' => {
                let next: String = chars.clone().take(10).collect();
                match &next[..] {
                    "Implements" => {
                        chars.by_ref().take(10).for_each(drop);
                        tokens.push(Token::Implements);
                    }
                    "isset" => {
                        chars.by_ref().take(5).for_each(drop);
                        tokens.push(Token::Isset);
                    }
                    "implode" => {
                        chars.by_ref().take(7).for_each(drop);
                        tokens.push(Token::Implode);
                    }
                    "in_array" => {
                        chars.by_ref().take(8).for_each(drop);
                        tokens.push(Token::InArray);
                    }
                    "If" => {
                        chars.by_ref().take(8).for_each(drop);
                        tokens.push(Token::If);
                    }
                    _ => {}
                }
            }
            'n' => {
                let next: String = chars.clone().take(4).collect();
                match &next[..] {
                    "null" => {
                        chars.by_ref().take(4).for_each(drop);
                        tokens.push(Token::Null);
                    }
                    "new" => {
                        chars.by_ref().take(3).for_each(drop);
                        tokens.push(Token::New);
                    }
                    _ => {}
                }
            }
            'p' => {
                let next: String = chars.clone().take(9).collect();
                match &next[..] {
                    "Protected" => {
                        chars.by_ref().take(9).for_each(drop);
                        tokens.push(Token::Protected);
                    }
                    "print_r" => {
                        chars.by_ref().take(7).for_each(drop);
                        tokens.push(Token::PrintR);
                    }
                    "print" => {
                        chars.by_ref().take(5).for_each(drop);
                        tokens.push(Token::Print);
                    }
                    "public" => {
                        chars.by_ref().take(6).for_each(drop);
                        tokens.push(Token::Public);
                    }
                    "private" => {
                        chars.by_ref().take(7).for_each(drop);
                        tokens.push(Token::Private);
                    }
                    _ => {}
                }
            }
            'r' => {
                let next: String = chars.clone().take(6).collect();
                if next == "return" {
                    chars.by_ref().take(6).for_each(drop);
                    tokens.push(Token::Return);
                }
            }
            's' => {
                let next: String = chars.clone().take(6).collect();
                match &next[..] {
                    "static" => {
                        chars.by_ref().take(6).for_each(drop);
                        tokens.push(Token::Static);
                    }
                    "switch" => {
                        chars.by_ref().take(6).for_each(drop);
                        tokens.push(Token::Switch);
                    }
                    "strlen" => {
                        chars.by_ref().take(6).for_each(drop);
                        tokens.push(Token::Strlen);
                    }
                    "strpos" => {
                        chars.by_ref().take(6).for_each(drop);
                        tokens.push(Token::Strpos);
                    }
                    "substr" => {
                        chars.by_ref().take(6).for_each(drop);
                        tokens.push(Token::Substr);
                    }
                    _ => {}
                }
            }
            't' => {
                let next: String = chars.clone().take(4).collect();
                if next == "true" {
                    chars.by_ref().take(4).for_each(drop);
                    tokens.push(Token::True);
                }
            }
            'v' => {
                let next: String = chars.clone().take(3).collect();
                if next == "var" {
                    chars.by_ref().take(3).for_each(drop);
                    tokens.push(Token::Var);
                }
            }
            'w' => {
                let next: String = chars.clone().take(5).collect();
                if next == "while" {
                    chars.by_ref().take(5).for_each(drop);
                    tokens.push(Token::While);
                }
            }
            '.' => { tokens.push(Token::Dot); chars.next(); }
            '=' => { tokens.push(Token::Equals); chars.next(); }
            ';' => { tokens.push(Token::Semicolon); chars.next(); }
            '(' => { tokens.push(Token::OpenParen); chars.next(); }
            ')' => { tokens.push(Token::CloseParen); chars.next(); }
            '{' => { tokens.push(Token::OpenBrace); chars.next(); }
            '}' => { tokens.push(Token::CloseBrace); chars.next(); }
            '+' => { tokens.push(Token::Plus); chars.next(); }
            '-' => { tokens.push(Token::Minus); chars.next(); }
            '*' => { tokens.push(Token::Multiply); chars.next(); }
            '/' => { tokens.push(Token::Divide); chars.next(); }
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
            _ if c.is_whitespace() => { chars.next(); }
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
            _ => { chars.next(); }
        }
    }

    tokens.push(Token::EOF);
    Ok(tokens)
}
