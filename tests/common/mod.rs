use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::time::Duration;

// Test configuration constants
pub const SERVER_URL: &str = "http://localhost:3000";

// Test URLs
pub const TEST_HTTPBIN_GET: &str = "https://httpbin.org/get";
pub const TEST_HTTPBIN_HTML: &str = "https://httpbin.org/html";
pub const TEST_AMAZON_URL: &str = "https://www.amazon.com/Star-Wars-Echo-Dot-bundle/dp/B0DZQ92XQZ/?_encoding=UTF8&pd_rd_w=J2REa&content-id=amzn1.sym.facdd3a9-7c82-4bfb-a2c8-ce73833c9be4&pf_rd_p=facdd3a9-7c82-4bfb-a2c8-ce73833c9be4&pf_rd_r=NGBMAN14SM5N4SCFJXGT&pd_rd_wg=5je2T&pd_rd_r=4ed5974f-7ae0-4192-9993-eaf90ae98cce&ref_=pd_hp_d_atf_dealz_sv&th=1";

// Test timeouts and durations
pub const SHORT_WAIT_MS: u64 = 1000;
pub const MEDIUM_WAIT_MS: u64 = 2000;
pub const LONG_WAIT_MS: u64 = 3000;
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Test session manager that handles session lifecycle
pub struct TestSession {
    pub id: String,
    pub client: Client,
}

impl TestSession {
    /// Create a new test session
    pub async fn new() -> Result<Self, String> {
        let client = Client::new();
        let session_id = create_session_with_client(&client).await?;
        Ok(Self {
            id: session_id,
            client,
        })
    }

    /// Clean up the session
    pub async fn cleanup(&self) {
        let _ = self
            .client
            .delete(&format!("{}/browser/session/{}", SERVER_URL, self.id))
            .send()
            .await;
    }

    /// Navigate to a URL using this session
    pub async fn navigate(&self, url: &str) -> Result<Value, String> {
        let response = self
            .client
            .post(&format!("{}/browser/navigate", SERVER_URL))
            .json(&json!({
                "session_id": self.id,
                "url": url
            }))
            .send()
            .await
            .map_err(|e| format!("Navigation request failed: {}", e))?;

        parse_json_response(response).await
    }

    /// Interact with the browser using this session
    pub async fn interact(&self, action: Value) -> Result<Value, String> {
        let response = self
            .client
            .post(&format!("{}/browser/interact", SERVER_URL))
            .json(&json!({
                "session_id": self.id,
                "action": action
            }))
            .send()
            .await
            .map_err(|e| format!("Interaction request failed: {}", e))?;

        parse_json_response(response).await
    }
}

impl Drop for TestSession {
    fn drop(&mut self) {
        // Schedule cleanup in the background
        let client = self.client.clone();
        let session_id = self.id.clone();
        tokio::spawn(async move {
            let _ = client
                .delete(&format!("{}/browser/session/{}", SERVER_URL, session_id))
                .send()
                .await;
        });
    }
}

/// Create a browser session and return the session ID
pub async fn create_session() -> Result<String, String> {
    let client = Client::new();
    create_session_with_client(&client).await
}

/// Create a browser session with a specific client
async fn create_session_with_client(client: &Client) -> Result<String, String> {
    let response = client
        .post(&format!("{}/browser/session", SERVER_URL))
        .send()
        .await
        .map_err(|e| format!("Failed to send session creation request: {}", e))?;

    let body = parse_json_response(response).await?;
    
    body["session_id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No session_id in response".to_string())
}

/// Check if the server is healthy and responding
pub async fn check_server_health() -> Result<(), String> {
    let client = Client::new();
    let response = client
        .get(&format!("{}/health", SERVER_URL))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;

    if response.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(format!("Server returned status: {}", response.status()))
    }
}

/// Parse a response as JSON and handle errors consistently
async fn parse_json_response(response: reqwest::Response) -> Result<Value, String> {
    let status = response.status();
    
    if !status.is_success() {
        let error_body: Value = response.json().await.unwrap_or_default();
        return Err(format!(
            "HTTP {} - {}",
            status,
            serde_json::to_string(&error_body).unwrap_or_default()
        ));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON response: {}", e))
}

/// Helper for making POST requests with consistent error handling
pub async fn post_json(
    client: &Client,
    endpoint: &str,
    payload: Value,
) -> Result<Value, String> {
    let response = client
        .post(&format!("{}{}", SERVER_URL, endpoint))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("POST request to {} failed: {}", endpoint, e))?;

    parse_json_response(response).await
}

/// Helper for making GET requests with consistent error handling
pub async fn get_request(client: &Client, endpoint: &str) -> Result<Value, String> {
    let response = client
        .get(&format!("{}{}", SERVER_URL, endpoint))
        .send()
        .await
        .map_err(|e| format!("GET request to {} failed: {}", endpoint, e))?;

    parse_json_response(response).await
}

/// Assert that a response indicates success
pub fn assert_success_response(body: &Value, operation: &str) {
    assert_eq!(
        body["success"], true,
        "{} should succeed. Got response: {}",
        operation,
        serde_json::to_string_pretty(body).unwrap_or_default()
    );
}

/// Assert that a response contains an error
pub fn assert_error_response(body: &Value, expected_status: u16) {
    assert_eq!(
        body["status"].as_u64().unwrap_or(0),
        expected_status as u64,
        "Expected status {}, got response: {}",
        expected_status,
        serde_json::to_string_pretty(body).unwrap_or_default()
    );
    assert!(
        body["error"].is_string(),
        "Error response should contain error message. Got: {}",
        serde_json::to_string_pretty(body).unwrap_or_default()
    );
}

/// Clean up any orphaned browser sessions
pub async fn cleanup_sessions() {
    let client = Client::new();
    let _ = client
        .post(&format!("{}/browser/sessions/cleanup", SERVER_URL))
        .send()
        .await;
}
