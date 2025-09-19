//! Integration test for playground execute endpoint

use actix_web::{test, App};

#[actix_web::test]
async fn test_execute_simple_echo() {
    // Configure minimal app with routes
    let app = test::init_service(
        App::new().configure(php_web::playground::init_routes)
    ).await;

    // Raw string payload form
    let req = test::TestRequest::post()
        .uri("/execute")
        .set_json(&"<?php echo 2+3; ?>".to_string())
        .to_request();
    let resp = test::call_and_read_body(&app, req).await;
    let body = String::from_utf8(resp.to_vec()).unwrap();
    assert!(body.contains("\"output\":\"5"), "Response body did not contain expected output: {}", body);

    // Object form payload
    let req2 = test::TestRequest::post()
        .uri("/api/execute")
        .set_json(&serde_json::json!({"code": "<?php echo 10/2; ?>"}))
        .to_request();
    let resp2 = test::call_and_read_body(&app, req2).await;
    let body2 = String::from_utf8(resp2.to_vec()).unwrap();
    assert!(body2.contains("\"output\":\"5"), "Response body did not contain expected output (object form): {}", body2);
}
