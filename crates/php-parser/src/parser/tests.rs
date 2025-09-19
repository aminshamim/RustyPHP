//! Unit tests for parser modules

use crate::parser::statements::StatementParser;
use crate::parser::expressions::ExpressionParser;  
use crate::parser::control_flow::ControlFlowParser;
use crate::ast::*;
use php_lexer::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_token_stream(tokens: Vec<Token>) -> (Peekable<IntoIter<Token>>, usize) {
        (tokens.into_iter().peekable(), 0)
    }

    #[test]
    fn test_statement_parser_echo() {
        let tokens = vec![
            Token::Echo,
            Token::String("Hello".to_string()),
            Token::Semicolon,
        ];
        let (mut token_stream, mut position) = create_token_stream(tokens);
        
        let result = StatementParser::parse_echo(&mut token_stream, &mut position);
        assert!(result.is_ok());
        
        if let Ok(Stmt::Echo(expr)) = result {
            assert!(matches!(expr, Expr::String(_)));
        } else {
            panic!("Expected echo statement");
        }
    }

    #[test]
    fn test_statement_parser_assignment() {
        let tokens = vec![
            Token::Variable("x".to_string()),
            Token::Equals,
            Token::Number(42.0),
            Token::Semicolon,
        ];
        let (mut token_stream, mut position) = create_token_stream(tokens);
        
        let result = StatementParser::parse_assignment_or_expression(&mut token_stream, &mut position);
        assert!(result.is_ok());
        
        if let Ok(Stmt::Assignment { variable, value }) = result {
            assert_eq!(variable, "x");
            assert!(matches!(value, Expr::Number(42.0)));
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_expression_parser_binary() {
        let tokens = vec![
            Token::Number(2.0),
            Token::Plus,
            Token::Number(3.0),
        ];
        let (mut token_stream, mut position) = create_token_stream(tokens);
        
        let result = ExpressionParser::parse_expression(&mut token_stream, &mut position);
        assert!(result.is_ok());
        
        if let Ok(Expr::Binary { left, op, right }) = result {
            assert!(matches!(*left, Expr::Number(2.0)));
            assert!(matches!(op, BinaryOp::Add));
            assert!(matches!(*right, Expr::Number(3.0)));
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_expression_precedence() {
        let tokens = vec![
            Token::Number(2.0),
            Token::Plus,
            Token::Number(3.0),
            Token::Multiply,
            Token::Number(4.0),
        ];
        let (mut token_stream, mut position) = create_token_stream(tokens);
        
        let result = ExpressionParser::parse_expression(&mut token_stream, &mut position);
        assert!(result.is_ok());
        
        // Should parse as 2 + (3 * 4) due to precedence
        if let Ok(Expr::Binary { left, op, right }) = result {
            assert!(matches!(*left, Expr::Number(2.0)));
            assert!(matches!(op, BinaryOp::Add));
            if let Expr::Binary { left: inner_left, op: inner_op, right: inner_right } = &*right {
                assert!(matches!(**inner_left, Expr::Number(3.0)));
                assert!(matches!(inner_op, BinaryOp::Multiply));
                assert!(matches!(**inner_right, Expr::Number(4.0)));
            } else {
                panic!("Expected nested binary expression");
            }
        }
    }

    #[test]
    fn test_control_flow_if() {
        let tokens = vec![
            Token::If,
            Token::OpenParen,
            Token::Variable("x".to_string()),
            Token::CloseParen,
            Token::Echo,
            Token::String("yes".to_string()),
            Token::Semicolon,
        ];
        let (mut token_stream, mut position) = create_token_stream(tokens);
        
        let result = ControlFlowParser::parse_if(&mut token_stream, &mut position);
        assert!(result.is_ok());
        
        if let Ok(Stmt::If { condition, then_stmt, else_stmt }) = result {
            assert!(matches!(condition, Expr::Variable(_)));
            assert!(matches!(*then_stmt, Stmt::Echo(_)));
            assert!(else_stmt.is_none());
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_control_flow_while() {
        let tokens = vec![
            Token::While,
            Token::OpenParen,
            Token::True,
            Token::CloseParen,
            Token::Echo,
            Token::String("loop".to_string()),
            Token::Semicolon,
        ];
        let (mut token_stream, mut position) = create_token_stream(tokens);
        
        let result = ControlFlowParser::parse_while(&mut token_stream, &mut position);
        assert!(result.is_ok());
        
        if let Ok(Stmt::While { condition, body }) = result {
            assert!(matches!(condition, Expr::Bool(true)));
            assert!(matches!(*body, Stmt::Echo(_)));
        } else {
            panic!("Expected while statement");
        }
    }
}
