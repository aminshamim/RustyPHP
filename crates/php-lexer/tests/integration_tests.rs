//! Integration tests for php-lexer

use php_lexer::*;

#[test]
fn test_basic_tokens() {
    let input = "<?php echo 'Hello World'; ?>";
    let tokens = lex(input).expect("Failed to lex input");
    
    // Expected: PhpOpen, Echo, String, Semicolon, PhpClose, EOF
    assert_eq!(tokens.len(), 6);
    assert!(matches!(tokens[0], Token::PhpOpen));
    assert!(matches!(tokens[1], Token::Echo));
    assert!(matches!(tokens[2], Token::String(_)));
    assert!(matches!(tokens[3], Token::Semicolon));
    assert!(matches!(tokens[4], Token::PhpClose));
    assert!(matches!(tokens[5], Token::EOF));
}

#[test]
fn test_variables() {
    let input = "<?php $name = 'John';";
    let tokens = lex(input).expect("Failed to lex input");
    
    assert!(matches!(tokens[1], Token::Variable(_)));
    assert!(matches!(tokens[2], Token::Equals));
    assert!(matches!(tokens[3], Token::String(_)));
}

#[test]
fn test_numbers() {
    let input = "<?php $age = 25; $price = 99.99;";
    let tokens = lex(input).expect("Failed to lex input");
    
    // Find number tokens
    let number_tokens: Vec<&Token> = tokens.iter()
        .filter(|t| matches!(t, Token::Number(_)))
        .collect();
    
    assert_eq!(number_tokens.len(), 2);
    if let Token::Number(n) = number_tokens[0] {
        assert_eq!(n, &25.0);
    }
    if let Token::Number(n) = number_tokens[1] {
        assert_eq!(n, &99.99);
    }
}

#[test]
fn test_comments() {
    let input = "<?php 
    // Single line comment
    echo 'test';
    /* Multi-line
       comment */";
    let tokens = lex(input).expect("Failed to lex input");
    
    // Comments should be filtered out, only meaningful tokens should remain
    let meaningful_tokens: Vec<&Token> = tokens.iter()
        .filter(|t| !matches!(t, Token::PhpOpen | Token::EOF))
        .collect();
    
    // Should have: Echo, String, Semicolon
    assert_eq!(meaningful_tokens.len(), 3);
}

#[test]
fn test_operators() {
    let input = "<?php $a = 1 + 2 * 3 / 4 - 5;";
    let tokens = lex(input).expect("Failed to lex input");
    
    let operator_tokens: Vec<&Token> = tokens.iter()
        .filter(|t| matches!(t, Token::Plus | Token::Minus | Token::Multiply | Token::Divide))
        .collect();
    
    assert_eq!(operator_tokens.len(), 4);
}

#[test]
fn test_keywords() {
    let input = "<?php if ($x) { return true; } else { return false; }";
    let tokens = lex(input).expect("Failed to lex input");
    
    let keyword_tokens: Vec<&Token> = tokens.iter()
        .filter(|t| matches!(t, Token::If | Token::Return | Token::Else | Token::True | Token::False))
        .collect();
    
    // Should have: if, return, true, else, return, false = 6 keywords
    assert_eq!(keyword_tokens.len(), 6);
}
