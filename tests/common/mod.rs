use reqwest;
use serde_json::Value;

// Server URL for external Docker server
pub const SERVER_URL: &str = "http://localhost:3000";

/// Create a browser session and return the session ID
pub async fn create_session() -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/browser/session", SERVER_URL))
        .send()
        .await
        .map_err(|e| format!("Failed to send session creation request: {}", e))?;

    if response.status() != reqwest::StatusCode::OK {
        let status = response.status();
        let error_body: Value = response.json().await.unwrap_or_default();
        return Err(format!(
            "HTTP {} - {}",
            status,
            serde_json::to_string(&error_body).unwrap_or_default()
        ));
    }

    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

    body["session_id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No session_id in response".to_string())
}

/// Check if the server is healthy and responding
pub async fn check_server_health() -> Result<(), String> {
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health", SERVER_URL))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;

    if response.status() == reqwest::StatusCode::OK {
        Ok(())
    } else {
        Err(format!("Server returned status: {}", response.status()))
    }
}
