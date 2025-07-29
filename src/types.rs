use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// Request/Response types for API endpoints

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatusResponse {
    pub session_id: String,
    pub active: bool,
    pub current_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NavigateRequest {
    pub session_id: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NavigateResponse {
    pub success: bool,
    pub current_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractionRequest {
    pub session_id: String,
    pub action: BrowserAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractionResponse {
    pub success: bool,
    pub result: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractRequest {
    pub session_id: String,
    pub selector: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractResponse {
    pub success: bool,
    pub data: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutomationRequest {
    pub session_id: String,
    pub task_description: String,
    pub target_url: Option<String>,
    pub context: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutomationResponse {
    pub success: bool,
    pub task_id: String,
    pub results: Vec<TaskResult>,
}

// Browser action types

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum BrowserAction {
    Click {
        selector: String,
    },
    Type {
        selector: String,
        text: String,
    },
    Wait {
        duration_ms: u64,
    },
    WaitForElement {
        selector: String,
        timeout_ms: Option<u64>,
    },
    Scroll {
        direction: ScrollDirection,
        pixels: Option<i32>,
    },
    Screenshot,
    GetPageSource,
    ExecuteScript {
        script: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

// Task planning and execution types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub steps: Vec<TaskStep>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub id: String,
    pub action: BrowserAction,
    pub description: String,
    pub expected_outcome: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub step_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

// Product extraction types

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductExtractionRequest {
    pub url: String,
    pub session_id: Option<String>, // Optional - will create temporary session if not provided
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductExtractionResponse {
    pub success: bool,
    pub product: Option<ProductInfo>,
    pub error: Option<String>,
    pub extraction_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<String>,
    pub availability: Option<String>,
    pub brand: Option<String>,
    pub rating: Option<String>,
    pub image_url: Option<String>,
    pub raw_data: Option<String>, // For debugging - contains the raw HTML that was analyzed
}

// Error handling

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Browser error: {0}")]
    BrowserError(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("MCP error: {0}")]
    MCPError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BrowserError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::SessionNotFound(session_id) => (
                StatusCode::NOT_FOUND,
                format!("Session {session_id} not found"),
            ),
            AppError::MCPError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::SerializationError(err) => (
                StatusCode::BAD_REQUEST,
                format!("Serialization error: {err}"),
            ),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
