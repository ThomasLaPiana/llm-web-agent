// Integration tests with improved organization and helper functions
//
// This file contains comprehensive integration tests for the LLM Web Agent.
// Tests are organized by functionality with consistent error handling and reduced code duplication.

mod common;

use common::{
    assert_error_response, assert_success_response, check_server_health, cleanup_sessions,
    get_request, post_json, TestSession, LONG_WAIT_MS, MEDIUM_WAIT_MS, SHORT_WAIT_MS,
    TEST_AMAZON_URL, TEST_HTTPBIN_GET, TEST_HTTPBIN_HTML,
};
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

// ===== BROWSER FUNCTIONALITY TESTS =====

/// Test server health endpoint
#[tokio::test]
async fn test_health_endpoint() {
    check_server_health()
        .await
        .expect("Server should be running");

    let client = Client::new();
    let body = get_request(&client, "/health")
        .await
        .expect("Health request should succeed");

    assert_eq!(body["status"], "healthy");
    assert!(body["active_sessions"].is_number());
    assert!(body["timestamp"].is_string());

    println!(
        "✅ Health endpoint test passed - {} active sessions",
        body["active_sessions"]
    );
}

/// Test navigation API error handling with invalid session
#[tokio::test]
async fn test_navigation_invalid_session() {
    let client = Client::new();
    
    let payload = json!({
        "session_id": "invalid-session-id",
        "url": TEST_HTTPBIN_GET
    });

    let response = client
        .post(&format!("{}/browser/navigate", common::SERVER_URL))
        .json(&payload)
        .send()
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body: Value = response.json().await.expect("Response should be JSON");
    assert_error_response(&body, 404);
    assert!(body["error"].as_str().unwrap().contains("Session"));

    println!("✅ Navigation API error handling test passed");
}

/// Test browser session creation
#[tokio::test]
async fn test_browser_session_creation() {
    let session = TestSession::new()
        .await
        .expect("Session creation should succeed");

    assert!(!session.id.is_empty());
    println!("✅ Browser session creation test passed - ID: {}", session.id);
}

/// Test wait action functionality
#[tokio::test]
async fn test_wait_action() {
    let session = TestSession::new()
        .await
        .expect("Browser session creation must succeed for this test");

    let start = std::time::Instant::now();

    let action = json!({
        "type": "Wait",
        "params": {
            "duration_ms": SHORT_WAIT_MS
        }
    });

    let body = session
        .interact(action)
        .await
        .expect("Wait request should succeed");

    let elapsed = start.elapsed();

    assert_success_response(&body, "Wait action");
    assert_eq!(body["result"], "Wait completed");

    // Verify it actually waited (with some tolerance)
    assert!(
        elapsed.as_millis() >= (SHORT_WAIT_MS - 50) as u128,
        "Should wait for at least the specified duration"
    );
    
    println!(
        "✅ Wait action test passed (actual wait: {}ms)",
        elapsed.as_millis()
    );
}

/// Test real browser navigation functionality
#[tokio::test]
async fn test_real_browser_navigation() {
    let session = TestSession::new()
        .await
        .expect("Browser session creation must succeed for this test");

    let body = session
        .navigate(TEST_HTTPBIN_GET)
        .await
        .expect("Navigation request should succeed");

    assert_success_response(&body, "Navigation");
    assert_eq!(body["current_url"], TEST_HTTPBIN_GET);

    // Wait for page to load
    sleep(Duration::from_millis(MEDIUM_WAIT_MS)).await;

    // Verify we can get page source and it contains expected content
    let source_action = json!({
        "type": "GetPageSource"
    });

    let source_body = session
        .interact(source_action)
        .await
        .expect("Page source request should succeed");

    assert_success_response(&source_body, "Page source extraction");

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
}

/// Test browser screenshot functionality
#[tokio::test]
async fn test_browser_screenshot() {
    let session = TestSession::new()
        .await
        .expect("Browser session creation must succeed for this test");

    // Navigate to a simple page first
    let nav_body = session
        .navigate(TEST_HTTPBIN_HTML)
        .await
        .expect("Navigation should succeed");

    assert_success_response(&nav_body, "Navigation to HTML page");

    // Wait for page to load
    sleep(Duration::from_millis(LONG_WAIT_MS)).await;

    // Take screenshot
    let screenshot_action = json!({
        "type": "Screenshot"
    });

    let body = session
        .interact(screenshot_action)
        .await
        .expect("Screenshot request should succeed");

    assert_success_response(&body, "Screenshot");

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
}

/// Test automation task fallback functionality
#[tokio::test]
async fn test_automation_task_fallback() {
    let session = TestSession::new()
        .await
        .expect("Browser session creation must succeed for this test");

    let client = Client::new();
    let payload = json!({
        "session_id": session.id,
        "task_description": "Take a screenshot of the page",
        "target_url": TEST_HTTPBIN_GET
    });

    let body = post_json(&client, "/automation/task", payload)
        .await
        .expect("Automation task request should succeed");

    assert_success_response(&body, "Automation task");
    assert!(body["task_id"].is_string(), "Should have a task ID");
    assert!(body["results"].is_array(), "Should have results array");

    let results = body["results"].as_array().expect("Results should be array");
    assert!(!results.is_empty(), "Should have at least one result");
    
    println!(
        "✅ Automation task fallback test passed ({} steps)",
        results.len()
    );
}

// ===== PRODUCT EXTRACTION TESTS =====

/// Test product extraction without an existing session (creates temporary session)
#[tokio::test]
async fn test_product_extraction_without_session() {
    let client = Client::new();

    let payload = json!({
        "url": TEST_HTTPBIN_HTML
    });

    let body = post_json(&client, "/product/information", payload)
        .await
        .expect("Product extraction request should succeed");

    assert_success_response(&body, "Product extraction");
    assert!(
        body["extraction_time_ms"].is_number(),
        "Should have extraction time"
    );
    assert!(body["product"].is_object(), "Should have product object");

    validate_product_structure(&body["product"]);

    println!(
        "✅ Product extraction test (no session) passed in {}ms",
        body["extraction_time_ms"].as_u64().unwrap_or(0)
    );

    cleanup_sessions().await;
}

/// Test product extraction with an existing session
#[tokio::test]
async fn test_product_extraction_with_existing_session() {
    let session = TestSession::new()
        .await
        .expect("Browser session creation must succeed for this test");

    let client = Client::new();
    let payload = json!({
        "url": TEST_HTTPBIN_HTML,
        "session_id": session.id
    });

    let body = post_json(&client, "/product/information", payload)
        .await
        .expect("Product extraction request should succeed");

    assert_success_response(&body, "Product extraction");
    assert!(
        body["extraction_time_ms"].is_number(),
        "Should have extraction time"
    );
    assert!(body["product"].is_object(), "Should have product object");

    validate_product_structure(&body["product"]);

    println!(
        "✅ Product extraction test (with session) passed in {}ms",
        body["extraction_time_ms"].as_u64().unwrap_or(0)
    );
}

/// Test product extraction with real Amazon URL
#[tokio::test]
async fn test_product_extraction_amazon_url() {
    let client = Client::new();

    let payload = json!({
        "url": TEST_AMAZON_URL
    });

    let body = post_json(&client, "/product/information", payload)
        .await
        .expect("Amazon product extraction request should succeed");

    assert_success_response(&body, "Amazon product extraction");
    assert!(
        body["extraction_time_ms"].is_number(),
        "Should have extraction time"
    );
    assert!(body["product"].is_object(), "Should have product object");

    let product = &body["product"];

    // Print the extracted product information for verification
    print_product_info(product);

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

    cleanup_sessions().await;
}

/// Test product extraction error handling with invalid session
#[tokio::test]
async fn test_product_extraction_invalid_session() {
    let client = Client::new();

    let payload = json!({
        "url": TEST_HTTPBIN_HTML,
        "session_id": "invalid-session-id"
    });

    let response = client
        .post(&format!("{}/product/information", common::SERVER_URL))
        .json(&payload)
        .send()
        .await
        .expect("Product extraction request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body: Value = response.json().await.expect("Response should be JSON");
    assert_error_response(&body, 404);
    assert!(body["error"].as_str().unwrap().contains("Session"));

    println!("✅ Product extraction invalid session error handling test passed");

    cleanup_sessions().await;
}

/// Test product extraction error handling with malformed requests
#[tokio::test]
async fn test_product_extraction_malformed_request() {
    let client = Client::new();

    // Test with missing URL
    let payload_no_url = json!({
        "session_id": "some-session"
    });

    let response = client
        .post(&format!("{}/product/information", common::SERVER_URL))
        .json(&payload_no_url)
        .send()
        .await
        .expect("Product extraction request should complete");

    assert!(
        response.status().is_client_error(),
        "Should get client error for malformed request"
    );

    // Test with empty request
    let empty_payload = json!({});

    let response2 = client
        .post(&format!("{}/product/information", common::SERVER_URL))
        .json(&empty_payload)
        .send()
        .await
        .expect("Product extraction request should complete");

    assert!(
        response2.status().is_client_error(),
        "Should get client error for empty request"
    );

    println!("✅ Product extraction malformed request error handling test passed");

    cleanup_sessions().await;
}

// ===== HELPER FUNCTIONS =====

/// Validate the basic structure of a product object
fn validate_product_structure(product: &Value) {
    // For httpbin, we won't get real product info, but the structure should be there
    assert!(product["name"].is_string() || product["name"].is_null());
    assert!(product["description"].is_string() || product["description"].is_null());
    assert!(product["price"].is_string() || product["price"].is_null());
}

/// Print product information for debugging/verification
fn print_product_info(product: &Value) {
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
}