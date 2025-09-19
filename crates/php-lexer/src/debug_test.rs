#[cfg(test)]
mod debug_test {
    use crate::lex;

    #[test]
    fn test_debug_increment_tokens() {
        let input = r#"<?php
$counter = 1;
$counter++;
"#;
        
        match lex(input) {
            Ok(tokens) => {
                for (i, token) in tokens.iter().enumerate() {
                    println!("{}: {:?}", i, token);
                }
                assert!(tokens.len() > 5);
            }
            Err(e) => {
                panic!("Lexer error: {:?}", e);
            }
        }
    }
}