//! LLM Web Agent - Core library providing web automation capabilities
//!
//! This library provides a web API for browser automation, product information extraction,
//! and AI-powered task automation using Llama and MCP (Model Context Protocol).

// Public module exports
pub mod browser;
pub mod llama_client;
pub mod mcp;
pub mod mcp_server;
pub mod types;

// Standard library imports
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// External crate imports
use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::info;

// Internal module imports
use crate::browser::BrowserSession;
use crate::llama_client::LlamaClient;
use crate::mcp_server::create_mcp_router;
use crate::types::*;

// ============================================================================
// Constants
// ============================================================================

/// Default page load wait time in seconds
const DEFAULT_PAGE_LOAD_WAIT_SECS: u64 = 2;

/// Default URL placeholder for session-based extraction
const DEFAULT_URL_PLACEHOLDER: &str = "https://example.com";

// ============================================================================
// Application State
// ============================================================================

/// Application state containing shared resources across all handlers
#[derive(Clone)]
pub struct AppState {
    /// Map of session IDs to browser sessions for persistent browsing
    pub browser_sessions: Arc<RwLock<HashMap<String, BrowserSession>>>,
    /// Shared Llama client for AI-powered operations
    pub llama_client: Arc<LlamaClient>,
}

impl AppState {
    /// Create a new application state with initialized clients
    pub async fn new() -> Result<Self> {
        let browser_sessions = Arc::new(RwLock::new(HashMap::new()));
        let llama_client = Arc::new(LlamaClient::new().await?);

        Ok(Self {
            browser_sessions,
            llama_client,
        })
    }

    /// Get a browser session by ID
    async fn get_browser_session(&self, session_id: &str) -> Result<(), AppError> {
        let sessions = self.browser_sessions.read().await;
        if sessions.contains_key(session_id) {
            Ok(())
        } else {
            Err(AppError::SessionNotFound(session_id.to_string()))
        }
    }
}

// ============================================================================
// Router Configuration
// ============================================================================

/// Create the main application router with all API endpoints
pub fn create_router() -> Router<AppState> {
    // Core API routes
    let api_router = Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        // Product information extraction (simplified endpoint)
        .route("/product/information", post(get_product_information))
        // Browser session management (for advanced users)
        .route("/browser/session", post(create_session))
        .route("/browser/session/:session_id", get(get_session))
        // Browser actions (for advanced users)
        .route("/browser/navigate", post(navigate))
        .route("/browser/extract", post(extract))
        // AI-powered automation
        .route("/automation/task", post(process_task));

    // Combine main API with MCP server routes
    Router::new()
        .nest("/api", api_router)
        .merge(create_mcp_router().with_state(Arc::new(crate::mcp_server::MCPServerState::new())))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a temporary browser session for one-off operations
async fn create_temporary_session() -> Result<BrowserSession, AppError> {
    BrowserSession::new()
        .await
        .map_err(|e| AppError::BrowserError(format!("Failed to create browser session: {}", e)))
}

/// Extract page content from a browser session
async fn get_page_content(session: &mut BrowserSession) -> Result<String, AppError> {
    session
        .interact(&BrowserAction::GetPageSource)
        .await
        .map_err(|e| AppError::BrowserError(format!("Failed to get page source: {}", e)))
}

/// Wait for page to load with default timeout
async fn wait_for_page_load() {
    tokio::time::sleep(Duration::from_secs(DEFAULT_PAGE_LOAD_WAIT_SECS)).await;
}

// ============================================================================
// API Handler Functions
// ============================================================================

/// Health check endpoint to verify service availability
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "message": "LLM Web Agent with Llama + MCP is running"
    }))
}

/// Extract product information from a URL using a temporary browser session
///
/// This endpoint creates a temporary browser session, navigates to the provided URL,
/// extracts the page content, and uses AI to extract structured product information.
async fn get_product_information(
    State(state): State<AppState>,
    Json(request): Json<ProductInformationRequest>,
) -> Result<Json<ProductInfo>, AppError> {
    info!("Getting product information for URL: {}", request.url);

    // Create a temporary browser session
    let mut session = create_temporary_session().await?;

    // Navigate to the URL
    session.navigate(&request.url).await.map_err(|e| {
        AppError::BrowserError(format!("Failed to navigate to {}: {}", request.url, e))
    })?;

    // Wait for page to load
    wait_for_page_load().await;

    // Get the page content
    let html_content = get_page_content(&mut session).await?;

    // Use Llama + MCP to extract product information
    let product_info = state
        .llama_client
        .extract_product_information(&request.url, &html_content)
        .await
        .map_err(|e| AppError::InternalError(format!("Product extraction failed: {}", e)))?;

    info!(
        "Successfully extracted product information from {}",
        request.url
    );
    Ok(Json(product_info))
    // Note: Session will be automatically cleaned up when it goes out of scope
}

/// Create a new persistent browser session
///
/// Creates a new browser session that can be reused across multiple requests.
/// Returns a session ID that should be used for subsequent operations.
async fn create_session(
    State(state): State<AppState>,
    Json(_request): Json<SessionCreateRequest>,
) -> Result<Json<SessionResponse>, AppError> {
    let session = create_temporary_session().await?;
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

/// Get information about an existing browser session
///
/// Returns session status and metadata for the specified session ID.
/// Note: Current URL retrieval is not yet implemented.
async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionResponse>, AppError> {
    state.get_browser_session(&session_id).await?;

    Ok(Json(SessionResponse {
        session_id: session_id.clone(),
        active: true,
        current_url: None, // TODO: Implement current URL retrieval from session
        created_at: None,
    }))
}

/// Navigate to a URL in an existing browser session
///
/// Directs the specified browser session to navigate to the given URL.
async fn navigate(
    State(state): State<AppState>,
    Json(request): Json<NavigateRequest>,
) -> Result<Json<NavigateResponse>, AppError> {
    // Verify session exists
    state.get_browser_session(&request.session_id).await?;

    let mut sessions = state.browser_sessions.write().await;
    let session = sessions
        .get_mut(&request.session_id)
        .ok_or_else(|| AppError::SessionNotFound(request.session_id.clone()))?;

    session
        .navigate(&request.url)
        .await
        .map_err(|e| AppError::BrowserError(format!("Navigation failed: {}", e)))?;

    info!(
        "Navigated to {} in session {}",
        request.url, request.session_id
    );

    Ok(Json(NavigateResponse {
        success: true,
        current_url: request.url.clone(),
    }))
}

/// Extract product information from the current page in a browser session
///
/// Uses AI to extract structured product information from the currently loaded page
/// in the specified browser session.
async fn extract(
    State(state): State<AppState>,
    Json(request): Json<ExtractRequest>,
) -> Result<Json<ProductInfo>, AppError> {
    // Verify session exists
    state.get_browser_session(&request.session_id).await?;

    let mut sessions = state.browser_sessions.write().await;
    let session = sessions
        .get_mut(&request.session_id)
        .ok_or_else(|| AppError::SessionNotFound(request.session_id.clone()))?;

    // Get the current page HTML
    let html_content = get_page_content(session).await?;

    // TODO: Get actual current URL from session instead of placeholder
    let current_url = DEFAULT_URL_PLACEHOLDER.to_string();

    // Use Llama + MCP to extract product information
    let product_info = state
        .llama_client
        .extract_product_information(&current_url, &html_content)
        .await
        .map_err(|e| AppError::InternalError(format!("Product extraction failed: {}", e)))?;

    info!("Successfully extracted product information using Llama + MCP");
    Ok(Json(product_info))
}

/// Process an AI-powered automation task
///
/// Analyzes the automation request and generates a task plan using AI.
/// The task plan can then be executed using the browser session APIs.
async fn process_task(
    State(state): State<AppState>,
    Json(request): Json<AutomationRequest>,
) -> Result<Json<TaskPlan>, AppError> {
    info!("Processing automation task with Llama + MCP");

    let task_plan = state
        .llama_client
        .process_automation_request(&request)
        .await
        .map_err(|e| AppError::InternalError(format!("Task processing failed: {}", e)))?;

    Ok(Json(task_plan))
}
