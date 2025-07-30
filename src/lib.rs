pub mod browser;
pub mod llama_client;
pub mod mcp;
pub mod mcp_server;
pub mod types;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::browser::BrowserSession;
use crate::llama_client::LlamaClient;
use crate::mcp_server::create_mcp_router;
use crate::types::*;

#[derive(Clone)]
pub struct AppState {
    pub browser_sessions: Arc<RwLock<HashMap<String, BrowserSession>>>,
    pub llama_client: Arc<LlamaClient>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let browser_sessions = Arc::new(RwLock::new(HashMap::new()));
        let llama_client = Arc::new(LlamaClient::new().await?);

        Ok(Self {
            browser_sessions,
            llama_client,
        })
    }
}

pub fn create_router() -> Router<AppState> {
    // Create the main API router
    let api_router = Router::new()
        .route("/health", get(health_check))
        // Browser session management
        .route("/browser/session", post(create_session))
        .route("/browser/session/:session_id", get(get_session))
        // Browser actions
        .route("/browser/navigate", post(navigate))
        .route("/browser/extract", post(extract))
        // AI automation
        .route("/automation/task", post(process_task));

    // Combine with MCP server routes
    Router::new()
        .nest("/api", api_router)
        .merge(create_mcp_router().with_state(Arc::new(crate::mcp_server::MCPServerState::new())))
}

// Handler functions
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "message": "LLM Web Agent with Llama + MCP is running"
    }))
}

async fn create_session(
    State(state): State<AppState>,
    Json(_request): Json<SessionCreateRequest>,
) -> Result<Json<SessionResponse>, StatusCode> {
    match BrowserSession::new().await {
        Ok(session) => {
            let session_id = uuid::Uuid::new_v4().to_string();
            state
                .browser_sessions
                .write()
                .await
                .insert(session_id.clone(), session);

            info!("Created new browser session: {}", session_id);
            Ok(Json(SessionResponse {
                session_id,
                active: true,
                current_url: None,
                created_at: Some(chrono::Utc::now().to_rfc3339()),
            }))
        }
        Err(e) => {
            warn!("Failed to create browser session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let sessions = state.browser_sessions.read().await;

    if sessions.contains_key(&session_id) {
        Ok(Json(SessionResponse {
            session_id: session_id.clone(),
            active: true,
            current_url: None, // TODO: Get current URL from session
            created_at: None,
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn navigate(
    State(state): State<AppState>,
    Json(request): Json<NavigateRequest>,
) -> Result<Json<NavigateResponse>, StatusCode> {
    let mut sessions = state.browser_sessions.write().await;

    if let Some(session) = sessions.get_mut(&request.session_id) {
        match session.navigate(&request.url).await {
            Ok(_) => {
                info!(
                    "Navigated to {} in session {}",
                    request.url, request.session_id
                );
                Ok(Json(NavigateResponse {
                    success: true,
                    current_url: request.url.clone(),
                }))
            }
            Err(e) => {
                warn!("Navigation failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn extract(
    State(state): State<AppState>,
    Json(request): Json<ExtractRequest>,
) -> Result<Json<ProductInfo>, StatusCode> {
    let mut sessions = state.browser_sessions.write().await;

    if let Some(session) = sessions.get_mut(&request.session_id) {
        // Get the current page HTML
        match session.interact(&BrowserAction::GetPageSource).await {
            Ok(html_content) => {
                let current_url = "https://example.com".to_string(); // TODO: Get actual URL

                // Use Llama + MCP to extract product information
                match state
                    .llama_client
                    .extract_product_information(&current_url, &html_content)
                    .await
                {
                    Ok(product_info) => {
                        info!("Successfully extracted product information using Llama + MCP");
                        Ok(Json(product_info))
                    }
                    Err(e) => {
                        warn!("Product extraction failed: {}", e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get page source: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn process_task(
    State(state): State<AppState>,
    Json(request): Json<AutomationRequest>,
) -> Result<Json<TaskPlan>, StatusCode> {
    info!("Processing automation task with Llama + MCP");

    match state
        .llama_client
        .process_automation_request(&request)
        .await
    {
        Ok(task_plan) => Ok(Json(task_plan)),
        Err(e) => {
            warn!("Task processing failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
