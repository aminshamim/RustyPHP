//! Integration tests for php-parser

use php_parser::*;
use php_parser::ast::*;
use php_lexer::{lex, Token};

#[test]
fn test_basic_echo() {
    let tokens = vec![
        Token::PhpOpen,
        Token::Echo,
        Token::String("Hello World".to_string()),
        Token::Semicolon,
        Token::EOF,
    ];
    
    let ast = parse(tokens).expect("Failed to parse");
    
    if let Stmt::Block(statements) = ast {
        assert_eq!(statements.len(), 1);
        assert!(matches!(statements[0], Stmt::Echo(_)));
    } else {
        panic!("Expected block statement");
    }
}

#[test]
fn test_variable_assignment() {
    let tokens = vec![
        Token::PhpOpen,
        Token::Variable("name".to_string()),
        Token::Equals,
        Token::String("John".to_string()),
        Token::Semicolon,
        Token::EOF,
    ];
    
    let ast = parse(tokens).expect("Failed to parse");
    
    if let Stmt::Block(statements) = ast {
        assert_eq!(statements.len(), 1);
        if let Stmt::Assignment { variable, .. } = &statements[0] {
            assert_eq!(variable, "name");
        } else {
            panic!("Expected assignment statement");
        }
    }
}

#[test]
fn test_arithmetic_expression() {
    let tokens = vec![
        Token::PhpOpen,
        Token::Echo,
        Token::Number(2.0),
        Token::Plus,
        Token::Number(3.0),
        Token::Multiply,
        Token::Number(4.0),
        Token::Semicolon,
        Token::EOF,
    ];
    
    let ast = parse(tokens).expect("Failed to parse");
    
    if let Stmt::Block(statements) = ast {
        assert_eq!(statements.len(), 1);
        if let Stmt::Echo(expr) = &statements[0] {
            // Should parse as 2 + (3 * 4) due to precedence
            if let Expr::Binary { left, op, right } = expr {
                assert!(matches!(op, BinaryOp::Add));
                assert!(matches!(**left, Expr::Number(2.0)));
                assert!(matches!(**right, Expr::Binary { .. }));
            }
        }
    }
}

#[test]
fn test_if_statement() {
    let tokens = vec![
        Token::PhpOpen,
        Token::If,
        Token::OpenParen,
        Token::Variable("x".to_string()),
        Token::CloseParen,
        Token::Echo,
        Token::String("yes".to_string()),
        Token::Semicolon,
        Token::EOF,
    ];
    
    let ast = parse(tokens).expect("Failed to parse");
    
    if let Stmt::Block(statements) = ast {
        assert_eq!(statements.len(), 1);
        assert!(matches!(statements[0], Stmt::If { .. }));
    }
}

#[test]
fn test_constant_definition() {
    let tokens = vec![
        Token::PhpOpen,
        Token::Const,
        Token::Identifier("API_URL".to_string()),
        Token::Equals,
        Token::String("https://api.example.com".to_string()),
        Token::Semicolon,
        Token::EOF,
    ];
    
    let ast = parse(tokens).expect("Failed to parse");
    
    if let Stmt::Block(statements) = ast {
        assert_eq!(statements.len(), 1);
        if let Stmt::ConstantDefinition { name, .. } = &statements[0] {
            assert_eq!(name, "API_URL");
        } else {
            panic!("Expected constant definition");
        }
    }
}

#[test]
fn test_multiple_statements() {
    let tokens = vec![
        Token::PhpOpen,
        Token::Variable("x".to_string()),
        Token::Equals,
        Token::Number(10.0),
        Token::Semicolon,
        Token::Echo,
        Token::Variable("x".to_string()),
        Token::Semicolon,
        Token::EOF,
    ];
    
    let ast = parse(tokens).expect("Failed to parse");
    
    if let Stmt::Block(statements) = ast {
        assert_eq!(statements.len(), 2);
        assert!(matches!(statements[0], Stmt::Assignment { .. }));
        assert!(matches!(statements[1], Stmt::Echo(_)));
    }
}

#[test]
fn test_full_integration() {
    let php_code = r#"<?php
        $name = "Alice";
        $age = 25;
        echo "Hello " . $name;
    "#;
    
    let tokens = lex(php_code).expect("Failed to lex");
    let ast = parse(tokens).expect("Failed to parse");
    
    if let Stmt::Block(statements) = ast {
        assert_eq!(statements.len(), 3); // 2 assignments, 1 echo
    }
}
