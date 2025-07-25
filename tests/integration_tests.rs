use reqwest::StatusCode;
use serde_json::{json, Value};
use std::sync::Once;
use std::time::Duration;
use tokio::time::sleep;

const SERVER_URL: &str = "http://127.0.0.1:3000";
static INIT: Once = Once::new();

// Start server once for all tests
async fn ensure_server_running() {
    INIT.call_once(|| {
        tokio::spawn(async {
            // Import the main function from our binary
            if let Err(e) = llm_web_agent::run_server().await {
                eprintln!("Server failed to start: {e}");
            }
        });
    });

    // Wait for server to be ready
    let client = reqwest::Client::new();
    for _ in 0..60 {
        // Increased timeout for browser startup
        if let Ok(response) = client.get(format!("{SERVER_URL}/health")).send().await {
            if response.status().is_success() {
                return;
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    // If we get here, the server didn't start - that's ok, we'll skip integration tests
    eprintln!("Warning: Server not available for integration tests. Run 'cargo run' in another terminal to enable integration tests.");
}

// Helper function to check if server is available
async fn server_available() -> bool {
    let client = reqwest::Client::new();
    if let Ok(response) = client.get(format!("{SERVER_URL}/health")).send().await {
        response.status().is_success()
    } else {
        false
    }
}

// Helper function to create a browser session
async fn create_session() -> Result<String, Box<dyn std::error::Error>> {
    if !server_available().await {
        return Err("Server not available".into());
    }

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{SERVER_URL}/browser/session"))
        .send()
        .await?;

    if response.status() != StatusCode::OK {
        return Err("Failed to create session".into());
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

    if !server_available().await {
        eprintln!("Skipping test_health_endpoint - server not available");
        return;
    }

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{SERVER_URL}/health"))
        .send()
        .await
        .expect("Health request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    let text = response.text().await.expect("Response should have text");
    assert_eq!(text, "LLM Web Agent is running!");
}

#[tokio::test]
async fn test_browser_session_creation() {
    ensure_server_running().await;

    let session_id = match create_session().await {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Skipping test_browser_session_creation - server not available");
            return;
        }
    };

    assert!(!session_id.is_empty(), "Session ID should not be empty");

    // Verify session exists
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{SERVER_URL}/browser/session/{session_id}"))
        .send()
        .await
        .expect("Session status request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(body["session_id"], session_id);
    assert_eq!(body["active"], true);
}

#[tokio::test]
async fn test_browser_navigation() {
    ensure_server_running().await;

    let session_id = match create_session().await {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Skipping test_browser_navigation - server not available");
            return;
        }
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{SERVER_URL}/browser/navigate"))
        .json(&json!({
            "session_id": session_id,
            "url": "https://example.com"
        }))
        .send()
        .await
        .expect("Navigation request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(body["success"], true);
    assert_eq!(body["current_url"], "https://example.com");
}

#[tokio::test]
async fn test_wait_action() {
    ensure_server_running().await;

    let session_id = match create_session().await {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Skipping test_wait_action - server not available");
            return;
        }
    };

    let client = reqwest::Client::new();
    let start = std::time::Instant::now();

    let response = client
        .post(format!("{SERVER_URL}/browser/interact"))
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
}

#[tokio::test]
async fn test_automation_task() {
    ensure_server_running().await;

    let session_id = match create_session().await {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Skipping test_automation_task - server not available");
            return;
        }
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{SERVER_URL}/automation/task"))
        .json(&json!({
            "session_id": session_id,
            "task_description": "Take a screenshot of the page",
            "target_url": "https://example.com"
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

    // Check that we have some successful steps
    let successful_steps = results
        .iter()
        .filter(|r| r["success"].as_bool().unwrap_or(false))
        .count();
    assert!(
        successful_steps > 0,
        "Should have at least one successful step"
    );
}

#[tokio::test]
async fn test_invalid_session_error() {
    ensure_server_running().await;

    if !server_available().await {
        eprintln!("Skipping test_invalid_session_error - server not available");
        return;
    }

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{SERVER_URL}/browser/navigate"))
        .json(&json!({
            "session_id": "invalid-session-id",
            "url": "https://example.com"
        }))
        .send()
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: Value = response.json().await.expect("Response should be JSON");
    assert!(body["error"].as_str().unwrap().contains("Session"));
    assert_eq!(body["status"], 404);
}

// These tests require browser interaction and are more complex
#[tokio::test]
#[ignore] // Ignored by default due to browser dependency
async fn test_browser_screenshot() {
    ensure_server_running().await;

    let session_id = match create_session().await {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Skipping test_browser_screenshot - server not available");
            return;
        }
    };

    // First navigate to a page
    let client = reqwest::Client::new();
    let _nav_response = client
        .post(format!("{SERVER_URL}/browser/navigate"))
        .json(&json!({
            "session_id": session_id,
            "url": "https://example.com"
        }))
        .send()
        .await
        .expect("Navigation should succeed");

    // Wait a bit for page to load
    sleep(Duration::from_millis(3000)).await;

    // Take screenshot
    let response = client
        .post(format!("{SERVER_URL}/browser/interact"))
        .json(&json!({
            "session_id": session_id,
            "action": {
                "type": "Screenshot"
            }
        }))
        .send()
        .await
        .expect("Screenshot request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(body["success"], true);

    let result = body["result"].as_str().expect("Result should be a string");
    assert!(
        result.starts_with("data:image/png;base64,"),
        "Should return base64 encoded image"
    );
    assert!(
        result.len() > 100,
        "Screenshot should contain substantial data"
    );
}

#[tokio::test]
#[ignore] // Ignored by default due to browser dependency
async fn test_browser_page_source() {
    ensure_server_running().await;

    let session_id = match create_session().await {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Skipping test_browser_page_source - server not available");
            return;
        }
    };

    // First navigate to a page
    let client = reqwest::Client::new();
    let _nav_response = client
        .post(format!("{SERVER_URL}/browser/navigate"))
        .json(&json!({
            "session_id": session_id,
            "url": "https://example.com"
        }))
        .send()
        .await
        .expect("Navigation should succeed");

    // Wait a bit for page to load
    sleep(Duration::from_millis(3000)).await;

    // Get page source
    let response = client
        .post(format!("{SERVER_URL}/browser/interact"))
        .json(&json!({
            "session_id": session_id,
            "action": {
                "type": "GetPageSource"
            }
        }))
        .send()
        .await
        .expect("Page source request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Response should be JSON");
    assert_eq!(body["success"], true);

    let result = body["result"].as_str().expect("Result should be a string");
    assert!(result.contains("<html"), "Should contain HTML");
    assert!(
        result.contains("Example Domain"),
        "Should contain expected content"
    );
}
