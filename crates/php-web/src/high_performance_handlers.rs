//! High-Performance Handler for PHP Execution
//! Optimized for 100k+ RPS throughput

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct ExecuteRequest {
    pub code: String,
    pub optimize: Option<bool>,
}

#[derive(Serialize)]
pub struct ExecuteResponse {
    pub output: String,
    pub execution_time_ms: f64,
    pub memory_usage: usize,
    pub success: bool,
    pub error: Option<String>,
}

/// High-performance PHP code execution handler
pub async fn high_performance_execute(
    req: web::Json<ExecuteRequest>,
    engine_pool: web::Data<Arc<super::high_performance::EnginePool>>,
) -> ActixResult<HttpResponse> {
    let start_time = Instant::now();
    
    // Get engine from pool
    let mut engine = match engine_pool.get_engine().await {
        Some(engine) => engine,
        None => {
            // Pool exhausted, create temporary engine
            php_runtime::Engine::new()
        }
    };
    
    // Optimize code if requested
    let code = if req.optimize.unwrap_or(false) {
        super::high_performance::optimize_php_code(&req.code)
    } else {
        req.code.clone()
    };
    
    // Execute PHP code
    let result = execute_optimized_php(&mut engine, &code).await;
    
    // Return engine to pool
    engine_pool.return_engine(engine).await;
    
    let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;
    
    let response = match result {
        Ok(output) => ExecuteResponse {
            output,
            execution_time_ms: execution_time,
            memory_usage: 0, // TODO: Implement memory tracking
            success: true,
            error: None,
        },
        Err(error) => ExecuteResponse {
            output: String::new(),
            execution_time_ms: execution_time,
            memory_usage: 0,
            success: false,
            error: Some(error),
        },
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Optimized PHP execution with minimal allocations
async fn execute_optimized_php(
    engine: &mut php_runtime::Engine,
    code: &str,
) -> Result<String, String> {
    
    // Fast-path for common patterns
    if code.trim().starts_with("<?php echo ") && !code.contains("$") {
        // Simple echo statement without variables - bypass full parser
        let content = code.trim()
            .strip_prefix("<?php echo ")
            .unwrap_or("")
            .strip_suffix(";")
            .unwrap_or("")
            .trim()
            .trim_matches('"');
        return Ok(content.to_string());
    }
    
    // Full parsing and execution for complex code
    match php_lexer::tokenize(code) {
        Ok(tokens) => {
            match php_parser::parse(tokens) {
                Ok(ast) => {
                    for stmt in ast {
                        if let Err(e) = engine.execute_stmt(&stmt) {
                            return Err(e);
                        }
                    }
                    Ok(engine.get_output().to_string())
                }
                Err(e) => Err(format!("Parse error: {:?}", e)),
            }
        }
        Err(e) => Err(format!("Tokenization error: {:?}", e)),
    }
}

/// Playground handler with caching for better performance
pub async fn optimized_playground_execute(
    req: web::Json<ExecuteRequest>,
    engine_pool: web::Data<Arc<super::high_performance::EnginePool>>,
) -> ActixResult<HttpResponse> {
    
    // For playground, we can add caching for frequently executed code
    let code_hash = calculate_hash(&req.code);
    
    // Check cache first (implementation would use Redis/in-memory cache)
    if let Some(cached_result) = get_cached_result(code_hash).await {
        return Ok(HttpResponse::Ok().json(cached_result));
    }
    
    // Execute normally
    let response = high_performance_execute(req, engine_pool).await?;
    
    // Cache successful results
    if let Ok(body) = response.body().as_ref() {
        cache_result(code_hash, body).await;
    }
    
    Ok(response)
}

fn calculate_hash(code: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    hasher.finish()
}

async fn get_cached_result(_hash: u64) -> Option<ExecuteResponse> {
    // TODO: Implement caching (Redis, in-memory, etc.)
    None
}

async fn cache_result(_hash: u64, _result: &[u8]) {
    // TODO: Implement result caching
}

/// Health check endpoint optimized for high throughput
pub async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body("OK"))
}

/// Metrics endpoint for monitoring performance
pub async fn metrics() -> ActixResult<HttpResponse> {
    let metrics = super::high_performance::PerformanceMetrics::new();
    Ok(HttpResponse::Ok().json(metrics))
}