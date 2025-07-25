use reqwest::StatusCode;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;
use std::time::Duration;
use tokio::time::sleep;

const SERVER_URL: &str = "http://127.0.0.1:3000";
static INIT: Once = Once::new();
static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

// Helper function to clean up Chrome processes and temp directories
fn cleanup_chrome() {
    // Kill any lingering Chrome processes
    let _ = std::process::Command::new("pkill")
        .arg("-f")
        .arg("chrome")
        .output();

    // Clean up the specific chromiumoxide-runner directory that causes conflicts
    let temp_dir = std::env::temp_dir();
    let chromium_dir = temp_dir.join("chromiumoxide-runner");
    if chromium_dir.exists() {
        let _ = std::fs::remove_dir_all(&chromium_dir);
    }

    // Wait longer for cleanup to complete and processes to fully terminate
    std::thread::sleep(std::time::Duration::from_millis(2000));
}

// Start server once for all tests
async fn ensure_server_running() {
    INIT.call_once(|| {
        // Start the server in a background thread
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match llm_web_agent::run_server().await {
                    Ok(_) => {
                        println!("✅ Test server started successfully");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to start test server: {}", e);
                    }
                }
            });
        });
    });

    // Wait for server to be ready with timeout
    let client = reqwest::Client::new();
    for i in 0..60 {
        // 30 seconds total
        if let Ok(response) = client.get(&format!("{}/health", SERVER_URL)).send().await {
            if response.status().is_success() {
                SERVER_RUNNING.store(true, Ordering::SeqCst);
                println!("✅ Test server is ready after {}ms", i * 500);
                return;
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    panic!("❌ Test server failed to start within 30 seconds");
}

// Helper function to create a browser session
async fn create_session() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/browser/session", SERVER_URL))
        .send()
        .await?;

    if response.status() != StatusCode::OK {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to create session: HTTP {} - {}", status, error_text).into());
    }

    let body: Value = response.json().await?;
    let session_id = body["session_id"]
        .as_str()
        .ok_or("No session_id in response")?
        .to_string();

    Ok(session_id)
}

#[tokio::test]
async fn test_health_endpoint() {
    ensure_server_running().await;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health", SERVER_URL))
        .send()
        .await
        .expect("Health request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    let text = response.text().await.expect("Response should have text");
    assert_eq!(text, "LLM Web Agent is running!");
    println!("✅ Health endpoint test passed");
}

#[tokio::test]
async fn test_browser_session_creation_api_only() {
    ensure_server_running().await;

    // Test the API contract - this test validates the API response format
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/browser/session", SERVER_URL))
        .send()
        .await
        .expect("Session creation request should succeed");

    // The API should either succeed or return a proper error response
    if response.status() == StatusCode::OK {
        let body: Value = response.json().await.expect("Response should be JSON");
        let session_id = body["session_id"].as_str().expect("Should have session_id");
        assert!(!session_id.is_empty(), "Session ID should not be empty");
        println!("✅ Browser session API test passed");
    } else {
        // If browser creation fails, we should get a proper error response
        assert!(
            response.status().is_client_error() || response.status().is_server_error(),
            "Should get proper error status, got: {}",
            response.status()
        );
        let error_body: Value = response
            .json()
            .await
            .expect("Error response should be JSON");
        assert!(error_body["error"].is_string(), "Should have error message");
        println!("✅ Browser session API error handling test passed");
    }
}

#[tokio::test]
async fn test_navigation_api_contract() {
    ensure_server_running().await;

    // Test navigation API without requiring actual browser
    let client = reqwest::Client::new();

    // Try to navigate with invalid session - should get proper error
    let response = client
        .post(&format!("{}/browser/navigate", SERVER_URL))
        .json(&json!({
            "session_id": "invalid-session-id",
            "url": "https://httpbin.org/get"
        }))
        .send()
        .await
        .expect("Navigation request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body: Value = response.json().await.expect("Response should be JSON");
    assert!(body["error"].as_str().unwrap().contains("Session"));
    println!("✅ Navigation API error handling test passed");
}

#[tokio::test]
async fn test_wait_action() {
    cleanup_chrome();
    ensure_server_running().await;

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
}

#[tokio::test]
async fn test_automation_task_fallback() {
    cleanup_chrome();
    ensure_server_running().await;

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
}

// Tests that require actual browser navigation
#[tokio::test]
async fn test_real_browser_navigation() {
    cleanup_chrome();
    ensure_server_running().await;

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
}

#[tokio::test]
async fn test_browser_screenshot() {
    cleanup_chrome();
    ensure_server_running().await;

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
}

#[tokio::test]
async fn test_invalid_session_error() {
    ensure_server_running().await;

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
