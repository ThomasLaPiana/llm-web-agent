use reqwest::StatusCode;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;
use std::time::Duration;
use tokio::time::sleep;

pub const SERVER_URL: &str = "http://127.0.0.1:3000";
static INIT: Once = Once::new();
static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static CLEANUP_GUARD: Once = Once::new();

// Cleanup guard that runs once at the end of all tests
struct TestCleanupGuard;

impl Drop for TestCleanupGuard {
    fn drop(&mut self) {
        println!("🧹 Running final test cleanup...");

        // Clean up any remaining browser sessions via API
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = reqwest::Client::new();
            if let Ok(_) = client
                .post(&format!("{}/browser/sessions/cleanup", SERVER_URL))
                .send()
                .await
            {
                println!("✅ Cleaned up browser sessions via API");
            }
        });

        // Kill any lingering Chrome processes
        let _ = std::process::Command::new("pkill")
            .arg("-f")
            .arg("chrome")
            .output();

        // Clean up temp directories
        let temp_dir = std::env::temp_dir();
        let chromium_dir = temp_dir.join("chromiumoxide-runner");
        if chromium_dir.exists() {
            let _ = std::fs::remove_dir_all(&chromium_dir);
        }

        println!("✅ Final test cleanup completed");
    }
}

// Initialize cleanup guard once
fn ensure_cleanup_guard() {
    CLEANUP_GUARD.call_once(|| {
        // Create a static cleanup guard that will drop when the program exits
        std::thread::spawn(|| {
            let _guard = TestCleanupGuard;
            // Keep this thread alive until the program exits
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        });
    });
}

// Start server once for all tests
pub async fn ensure_server_running() {
    ensure_cleanup_guard();

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
pub async fn create_session() -> Result<String, Box<dyn std::error::Error>> {
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
