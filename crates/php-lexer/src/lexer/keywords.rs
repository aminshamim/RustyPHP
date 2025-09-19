//! Keyword recognition for PHP lexer
//!
//! This module handles PHP keyword and identifier recognition.

use std::collections::HashMap;
use crate::token::Token;

/// Keyword and identifier handling functionality
pub struct KeywordHandler {
    keywords: HashMap<&'static str, Token>,
}

impl KeywordHandler {
    /// Create a new keyword handler with all PHP keywords
    pub fn new() -> Self {
        let mut keywords = HashMap::new();
        
        // Language constructs
        keywords.insert("echo", Token::Echo);
        keywords.insert("print", Token::Print);
        keywords.insert("if", Token::If);
        keywords.insert("else", Token::Else);
        keywords.insert("while", Token::While);
        keywords.insert("for", Token::For);
        keywords.insert("function", Token::Function);
        keywords.insert("return", Token::Return);
        keywords.insert("class", Token::Class);
        keywords.insert("extends", Token::Extends);
        keywords.insert("implements", Token::Implements);
        keywords.insert("new", Token::New);
        keywords.insert("public", Token::Public);
        keywords.insert("private", Token::Private);
        keywords.insert("protected", Token::Protected);
        keywords.insert("static", Token::Static);
        keywords.insert("var", Token::Var);
        keywords.insert("const", Token::Const);
        keywords.insert("true", Token::True);
        keywords.insert("false", Token::False);
        keywords.insert("null", Token::Null);
        keywords.insert("isset", Token::Isset);
        keywords.insert("empty", Token::Empty);
        keywords.insert("switch", Token::Switch);
        keywords.insert("case", Token::Case);
        keywords.insert("default", Token::Default);
        keywords.insert("break", Token::Break);
        keywords.insert("continue", Token::Continue);
        keywords.insert("do", Token::Do);
        
        // Built-in functions
        keywords.insert("print_r", Token::PrintR);
        keywords.insert("strlen", Token::Strlen);
        keywords.insert("strpos", Token::Strpos);
        keywords.insert("substr", Token::Substr);
        keywords.insert("array_push", Token::ArrayPush);
        keywords.insert("array_pop", Token::ArrayPop);
        keywords.insert("array_merge", Token::ArrayMerge);
        keywords.insert("in_array", Token::InArray);
        keywords.insert("explode", Token::Explode);
        keywords.insert("implode", Token::Implode);
        keywords.insert("count", Token::Count);
        
        Self { keywords }
    }

    /// Look up a keyword or return identifier token
    pub fn lookup_keyword(&self, word: &str) -> Token {
        self.keywords.get(word)
            .cloned()
            .unwrap_or_else(|| Token::Identifier(word.to_string()))
    }
}

impl Default for KeywordHandler {
    fn default() -> Self {
        Self::new()
    }
}
