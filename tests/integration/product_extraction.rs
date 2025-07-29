use reqwest::StatusCode;
use serde_json::{json, Value};

mod common;
use common::{create_session, ensure_server_running, SERVER_URL};

#[tokio::test]
async fn test_product_extraction_without_session() {
    ensure_server_running().await;

    let client = reqwest::Client::new();

    // Test with a simple product page (httpbin for basic test)
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
}

#[tokio::test]
async fn test_product_extraction_with_existing_session() {
    ensure_server_running().await;

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
}

#[tokio::test]
async fn test_product_extraction_amazon_url() {
    ensure_server_running().await;

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
}

#[tokio::test]
async fn test_product_extraction_invalid_session() {
    ensure_server_running().await;

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
}

#[tokio::test]
async fn test_product_extraction_malformed_request() {
    ensure_server_running().await;

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
}
