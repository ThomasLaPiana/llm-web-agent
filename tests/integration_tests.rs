mod common;
use common::{check_server_health, create_session, SERVER_URL};
use reqwest::StatusCode;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

// ===== BROWSER TESTS =====

#[tokio::test]
async fn test_health_endpoint() {
    // Verify server is running (external Docker server)
    check_server_health()
        .await
        .expect("Server should be running");

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health", SERVER_URL))
        .send()
        .await
        .expect("Health request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(body["status"], "healthy");
    assert!(body["active_sessions"].is_number());
    assert!(body["timestamp"].is_string());

    println!(
        "✅ Health endpoint test passed - {} active sessions",
        body["active_sessions"]
    );
}

#[tokio::test]
async fn test_navigation_api_contract() {
    let client = reqwest::Client::new();

    // Test with invalid session ID
    let response = client
        .post(&format!("{}/browser/navigate", SERVER_URL))
        .json(&json!({
            "session_id": "invalid-session-id",
            "url": "https://httpbin.org/get"
        }))
        .send()
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body: Value = response.json().await.expect("Response should be JSON");
    assert!(body["error"].as_str().unwrap().contains("Session"));
    assert_eq!(body["status"], 404);

    println!("✅ Navigation API error handling test passed");
}

#[tokio::test]
async fn test_browser_session_creation_api_only() {
    // Just test the API response without actually using the browser
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/browser/session", SERVER_URL))
        .send()
        .await
        .expect("Session creation should succeed");

    if response.status() != StatusCode::OK {
        let status = response.status();
        let error_body: Value = response.json().await.unwrap_or_default();
        panic!(
            "Session creation failed with status {}: {:?}",
            status, error_body
        );
    }

    let body: Value = response.json().await.expect("Response should be JSON");
    assert!(body["session_id"].is_string());
    let session_id = body["session_id"].as_str().unwrap();
    assert!(!session_id.is_empty());

    // Clean up this specific session
    let _ = client
        .delete(&format!("{}/browser/session/{}", SERVER_URL, session_id))
        .send()
        .await;

    println!("✅ Browser session API test passed");
}

#[tokio::test]
async fn test_wait_action() {
    let session_id = create_session()
        .await
        .expect("Browser session creation must succeed for this test");

    let client = reqwest::Client::new();
    let start = std::time::Instant::now();

    let response = client
        .post(&format!("{}/browser/interact", SERVER_URL))
        .json(&json!({
            "session_id": session_id,
            "action": {
                "type": "Wait",
                "params": {
                    "duration_ms": 1000
                }
            }
        }))
        .send()
        .await
        .expect("Wait request should succeed");

    let elapsed = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(body["success"], true);
    assert_eq!(body["result"], "Wait completed");

    // Verify it actually waited (with some tolerance)
    assert!(
        elapsed.as_millis() >= 950,
        "Should wait for at least the specified duration"
    );
    println!(
        "✅ Wait action test passed (actual wait: {}ms)",
        elapsed.as_millis()
    );

    // Clean up this specific session
    let _ = client
        .delete(&format!("{}/browser/session/{}", SERVER_URL, session_id))
        .send()
        .await;
}

#[tokio::test]
async fn test_automation_task_fallback() {
    let session_id = create_session()
        .await
        .expect("Browser session creation must succeed for this test");

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/automation/task", SERVER_URL))
        .json(&json!({
            "session_id": session_id,
            "task_description": "Take a screenshot of the page",
            "target_url": "https://httpbin.org/get"
        }))
        .send()
        .await
        .expect("Automation task request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(body["success"], true);
    assert!(body["task_id"].is_string(), "Should have a task ID");
    assert!(body["results"].is_array(), "Should have results array");

    let results = body["results"].as_array().expect("Results should be array");
    assert!(!results.is_empty(), "Should have at least one result");
    println!(
        "✅ Automation task fallback test passed ({} steps)",
        results.len()
    );

    // Clean up this specific session
    let _ = client
        .delete(&format!("{}/browser/session/{}", SERVER_URL, session_id))
        .send()
        .await;
}

// Tests that require actual browser navigation
#[tokio::test]
async fn test_real_browser_navigation() {
    let session_id = create_session()
        .await
        .expect("Browser session creation must succeed for this test");

    // Use a reliable test endpoint
    let test_url = "https://httpbin.org/get";

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/browser/navigate", SERVER_URL))
        .json(&json!({
            "session_id": session_id,
            "url": test_url
        }))
        .send()
        .await
        .expect("Navigation request should succeed");

    if response.status() != StatusCode::OK {
        let status = response.status();
        let error_body: Value = response.json().await.unwrap_or_default();
        panic!("Navigation failed with status {}: {:?}", status, error_body);
    }

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(body["success"], true, "Navigation must be successful");
    assert_eq!(body["current_url"], test_url, "URL must match");

    // Wait for page to load
    sleep(Duration::from_millis(2000)).await;

    // Verify we can get page source and it contains expected content
    let source_response = client
        .post(&format!("{}/browser/interact", SERVER_URL))
        .json(&json!({
            "session_id": session_id,
            "action": {
                "type": "GetPageSource"
            }
        }))
        .send()
        .await
        .expect("Page source request should succeed");

    assert_eq!(
        source_response.status(),
        StatusCode::OK,
        "Page source request must succeed"
    );
    let source_body: Value = source_response
        .json()
        .await
        .expect("Response should be JSON");
    assert_eq!(
        source_body["success"], true,
        "Page source extraction must succeed"
    );

    let page_source = source_body["result"]
        .as_str()
        .expect("Should have page source");

    // httpbin.org/get returns JSON with request info
    assert!(
        page_source.contains("httpbin.org") || page_source.contains("headers"),
        "Page should contain httpbin content, got: {}",
        &page_source[..std::cmp::min(200, page_source.len())]
    );

    println!("✅ Real browser navigation test passed");

    // Clean up this specific session
    let _ = client
        .delete(&format!("{}/browser/session/{}", SERVER_URL, session_id))
        .send()
        .await;
}

#[tokio::test]
async fn test_browser_screenshot() {
    let session_id = create_session()
        .await
        .expect("Browser session creation must succeed for this test");

    // Navigate to a simple page first
    let client = reqwest::Client::new();
    let nav_response = client
        .post(&format!("{}/browser/navigate", SERVER_URL))
        .json(&json!({
            "session_id": session_id,
            "url": "https://httpbin.org/html"
        }))
        .send()
        .await
        .expect("Navigation should succeed");

    assert_eq!(
        nav_response.status(),
        StatusCode::OK,
        "Navigation must succeed"
    );

    // Wait for page to load
    sleep(Duration::from_millis(3000)).await;

    // Take screenshot
    let response = client
        .post(&format!("{}/browser/interact", SERVER_URL))
        .json(&json!({
            "session_id": session_id,
            "action": {
                "type": "Screenshot"
            }
        }))
        .send()
        .await
        .expect("Screenshot request should succeed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Screenshot request must succeed"
    );

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(body["success"], true, "Screenshot must be successful");

    let result = body["result"].as_str().expect("Result should be a string");
    assert!(
        result.starts_with("data:image/png;base64,"),
        "Should return base64 encoded image"
    );
    assert!(
        result.len() > 1000,
        "Screenshot should contain substantial data"
    );

    println!(
        "✅ Browser screenshot test passed ({}KB)",
        result.len() / 1024
    );

    // Clean up this specific session
    let _ = client
        .delete(&format!("{}/browser/session/{}", SERVER_URL, session_id))
        .send()
        .await;
}

#[tokio::test]
async fn test_invalid_session_error() {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/browser/navigate", SERVER_URL))
        .json(&json!({
            "session_id": "invalid-session-id",
            "url": "https://httpbin.org/get"
        }))
        .send()
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body: Value = response.json().await.expect("Response should be JSON");
    assert!(body["error"].as_str().unwrap().contains("Session"));
    assert_eq!(body["status"], 404);

    println!("✅ Invalid session error handling test passed");
}

// ===== PRODUCT EXTRACTION TESTS =====

#[tokio::test]
async fn test_product_extraction_without_session() {
    let client = reqwest::Client::new();

    // Test without session ID (should create temporary session)
    let response = client
        .post(&format!("{}/product/information", SERVER_URL))
        .json(&json!({
            "url": "https://httpbin.org/html"
        }))
        .send()
        .await
        .expect("Product extraction request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(body["success"], true, "Product extraction should succeed");
    assert!(
        body["extraction_time_ms"].is_number(),
        "Should have extraction time"
    );
    assert!(body["product"].is_object(), "Should have product object");

    let product = &body["product"];
    // For httpbin, we won't get real product info, but the structure should be there
    assert!(product["name"].is_string() || product["name"].is_null());
    assert!(product["description"].is_string() || product["description"].is_null());
    assert!(product["price"].is_string() || product["price"].is_null());

    println!(
        "✅ Product extraction test (no session) passed in {}ms",
        body["extraction_time_ms"].as_u64().unwrap_or(0)
    );

    // Clean up any browser sessions created during this test
    let _ = client
        .post(&format!("{}/browser/sessions/cleanup", SERVER_URL))
        .send()
        .await;
}

#[tokio::test]
async fn test_product_extraction_with_existing_session() {
    let session_id = create_session()
        .await
        .expect("Browser session creation must succeed for this test");

    let client = reqwest::Client::new();

    // Test with existing session
    let response = client
        .post(&format!("{}/product/information", SERVER_URL))
        .json(&json!({
            "url": "https://httpbin.org/html",
            "session_id": session_id
        }))
        .send()
        .await
        .expect("Product extraction request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(body["success"], true, "Product extraction should succeed");
    assert!(
        body["extraction_time_ms"].is_number(),
        "Should have extraction time"
    );
    assert!(body["product"].is_object(), "Should have product object");

    println!(
        "✅ Product extraction test (with session) passed in {}ms",
        body["extraction_time_ms"].as_u64().unwrap_or(0)
    );

    // Clean up this specific session
    let _ = client
        .delete(&format!("{}/browser/session/{}", SERVER_URL, session_id))
        .send()
        .await;
}

#[tokio::test]
async fn test_product_extraction_amazon_url() {
    let client = reqwest::Client::new();

    // Test with the real Amazon URL provided by the user
    let amazon_url = "https://www.amazon.com/Star-Wars-Echo-Dot-bundle/dp/B0DZQ92XQZ/?_encoding=UTF8&pd_rd_w=J2REa&content-id=amzn1.sym.facdd3a9-7c82-4bfb-a2c8-ce73833c9be4&pf_rd_p=facdd3a9-7c82-4bfb-a2c8-ce73833c9be4&pf_rd_r=NGBMAN14SM5N4SCFJXGT&pd_rd_wg=5je2T&pd_rd_r=4ed5974f-7ae0-4192-9993-eaf90ae98cce&ref_=pd_hp_d_atf_dealz_sv&th=1";

    let response = client
        .post(&format!("{}/product/information", SERVER_URL))
        .json(&json!({
            "url": amazon_url
        }))
        .send()
        .await
        .expect("Amazon product extraction request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(
        body["success"], true,
        "Amazon product extraction should succeed"
    );
    assert!(
        body["extraction_time_ms"].is_number(),
        "Should have extraction time"
    );
    assert!(body["product"].is_object(), "Should have product object");

    let product = &body["product"];

    // Print the extracted product information for verification
    println!("📦 Extracted Product Information:");
    if let Some(name) = product["name"].as_str() {
        println!("   Name: {}", name);
    }
    if let Some(description) = product["description"].as_str() {
        println!("   Description: {}", description);
    }
    if let Some(price) = product["price"].as_str() {
        println!("   Price: {}", price);
    }
    if let Some(availability) = product["availability"].as_str() {
        println!("   Availability: {}", availability);
    }
    if let Some(brand) = product["brand"].as_str() {
        println!("   Brand: {}", brand);
    }
    if let Some(rating) = product["rating"].as_str() {
        println!("   Rating: {}", rating);
    }
    if let Some(image_url) = product["image_url"].as_str() {
        println!("   Image URL: {}", image_url);
    }

    // Verify that we got at least some product information
    // Note: This test might not extract perfect data depending on LLM availability,
    // but we should at least get the structure and some attempt at extraction
    assert!(
        product["name"].is_string()
            || product["description"].is_string()
            || product["price"].is_string(),
        "Should extract at least one piece of product information from Amazon page"
    );

    println!(
        "✅ Amazon product extraction test passed in {}ms",
        body["extraction_time_ms"].as_u64().unwrap_or(0)
    );

    // Clean up any temporary sessions (this test doesn't use a specific session)
    let _ = client
        .post(&format!("{}/browser/sessions/cleanup", SERVER_URL))
        .send()
        .await;
}

#[tokio::test]
async fn test_product_extraction_invalid_session() {
    let client = reqwest::Client::new();

    // Test with invalid session ID
    let response = client
        .post(&format!("{}/product/information", SERVER_URL))
        .json(&json!({
            "url": "https://httpbin.org/html",
            "session_id": "invalid-session-id"
        }))
        .send()
        .await
        .expect("Product extraction request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body: Value = response.json().await.expect("Response should be JSON");
    assert!(body["error"].as_str().unwrap().contains("Session"));
    assert_eq!(body["status"], 404);

    println!("✅ Product extraction invalid session error handling test passed");

    // Clean up browser sessions (just in case)
    let _ = client
        .post(&format!("{}/browser/sessions/cleanup", SERVER_URL))
        .send()
        .await;
}

#[tokio::test]
async fn test_product_extraction_malformed_request() {
    let client = reqwest::Client::new();

    // Test with missing URL
    let response = client
        .post(&format!("{}/product/information", SERVER_URL))
        .json(&json!({
            "session_id": "some-session"
        }))
        .send()
        .await
        .expect("Product extraction request should complete");

    // Should get a 400 Bad Request for malformed JSON
    assert!(
        response.status().is_client_error(),
        "Should get client error for malformed request"
    );

    // Test with empty request
    let response2 = client
        .post(&format!("{}/product/information", SERVER_URL))
        .json(&json!({}))
        .send()
        .await
        .expect("Product extraction request should complete");

    assert!(
        response2.status().is_client_error(),
        "Should get client error for empty request"
    );

    println!("✅ Product extraction malformed request error handling test passed");

    // Clean up browser sessions (just in case)
    let _ = client
        .post(&format!("{}/browser/sessions/cleanup", SERVER_URL))
        .send()
        .await;
}
