use axum::{
    extract::State,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use uuid::Uuid;

pub mod browser;
pub mod mcp;
pub mod types;

pub use types::*;

#[derive(Clone)]
pub struct AppState {
    browser_sessions: Arc<RwLock<HashMap<String, browser::BrowserSession>>>,
    mcp_client: Arc<mcp::MCPClient>,
}

pub async fn run_server() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("Starting LLM Web Agent server...");

    // Initialize MCP client
    let mcp_client = Arc::new(mcp::MCPClient::new().await?);

    // Create application state
    let state = AppState {
        browser_sessions: Arc::new(RwLock::new(HashMap::new())),
        mcp_client,
    };

    // Build our application with routes
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/browser/session", post(create_browser_session))
        .route("/browser/session/:session_id", get(get_browser_session))
        .route(
            "/browser/session/:session_id",
            delete(cleanup_browser_session),
        )
        .route("/browser/sessions/cleanup", post(cleanup_all_sessions))
        .route("/browser/navigate", post(navigate_to_url))
        .route("/browser/interact", post(interact_with_page))
        .route("/browser/extract", post(extract_page_data))
        .route("/automation/task", post(execute_automation_task))
        .route("/product/information", post(extract_product_information))
        .route("/debug/page-content", post(debug_page_content))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    let sessions = state.browser_sessions.read().await;
    let session_count = sessions.len();

    // Get memory usage if available
    let memory_info = get_memory_info();

    Json(json!({
        "status": "healthy",
        "message": "LLM Web Agent is running!",
        "active_sessions": session_count,
        "memory_usage_mb": memory_info,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

fn get_memory_info() -> Option<u64> {
    // Try to get memory usage on Unix systems
    #[cfg(unix)]
    {
        use std::fs;
        if let Ok(contents) = fs::read_to_string("/proc/self/status") {
            for line in contents.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return Some(kb / 1024); // Convert KB to MB
                        }
                    }
                }
            }
        }
    }
    None
}

// New cleanup endpoints
async fn cleanup_browser_session(
    State(state): State<AppState>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut sessions = state.browser_sessions.write().await;

    if sessions.remove(&session_id).is_some() {
        info!("Cleaned up browser session: {}", session_id);
        Ok(Json(json!({
            "success": true,
            "message": format!("Session {} cleaned up successfully", session_id),
            "remaining_sessions": sessions.len()
        })))
    } else {
        Err(AppError::SessionNotFound(session_id))
    }
}

async fn cleanup_all_sessions(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut sessions = state.browser_sessions.write().await;
    let cleaned_count = sessions.len();
    sessions.clear();

    info!("Cleaned up all {} browser sessions", cleaned_count);
    Json(json!({
        "success": true,
        "message": format!("Cleaned up {} browser sessions", cleaned_count),
        "remaining_sessions": 0
    }))
}

async fn create_browser_session(
    State(state): State<AppState>,
) -> Result<Json<CreateSessionResponse>, AppError> {
    let session_id = Uuid::new_v4().to_string();

    let browser_session = browser::BrowserSession::new()
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    state
        .browser_sessions
        .write()
        .await
        .insert(session_id.clone(), browser_session);

    Ok(Json(CreateSessionResponse { session_id }))
}

async fn get_browser_session(
    State(state): State<AppState>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Result<Json<SessionStatusResponse>, AppError> {
    let sessions = state.browser_sessions.read().await;

    if sessions.contains_key(&session_id) {
        Ok(Json(SessionStatusResponse {
            session_id,
            active: true,
            current_url: None, // TODO: Get current URL from browser session
        }))
    } else {
        Err(AppError::SessionNotFound(session_id))
    }
}

async fn navigate_to_url(
    State(state): State<AppState>,
    Json(request): Json<NavigateRequest>,
) -> Result<Json<NavigateResponse>, AppError> {
    let mut sessions = state.browser_sessions.write().await;

    let session = sessions
        .get_mut(&request.session_id)
        .ok_or_else(|| AppError::SessionNotFound(request.session_id.clone()))?;

    session
        .navigate(&request.url)
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    Ok(Json(NavigateResponse {
        success: true,
        current_url: request.url,
    }))
}

async fn interact_with_page(
    State(state): State<AppState>,
    Json(request): Json<InteractionRequest>,
) -> Result<Json<InteractionResponse>, AppError> {
    let mut sessions = state.browser_sessions.write().await;

    let session = sessions
        .get_mut(&request.session_id)
        .ok_or_else(|| AppError::SessionNotFound(request.session_id.clone()))?;

    let result = session
        .interact(&request.action)
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    Ok(Json(InteractionResponse {
        success: true,
        result: Some(result),
    }))
}

async fn extract_page_data(
    State(state): State<AppState>,
    Json(request): Json<ExtractRequest>,
) -> Result<Json<ExtractResponse>, AppError> {
    let sessions = state.browser_sessions.read().await;

    let session = sessions
        .get(&request.session_id)
        .ok_or_else(|| AppError::SessionNotFound(request.session_id.clone()))?;

    let data = session
        .extract_data(&request.selector)
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    Ok(Json(ExtractResponse {
        success: true,
        data,
    }))
}

async fn execute_automation_task(
    State(state): State<AppState>,
    Json(request): Json<AutomationRequest>,
) -> Result<Json<AutomationResponse>, AppError> {
    // Use MCP client to process the automation request with LLM
    let task_plan = state
        .mcp_client
        .process_automation_request(&request)
        .await
        .map_err(|e| AppError::MCPError(e.to_string()))?;

    // Execute the planned actions using browser automation
    let mut sessions = state.browser_sessions.write().await;
    let session = sessions
        .get_mut(&request.session_id)
        .ok_or_else(|| AppError::SessionNotFound(request.session_id.clone()))?;

    let results = session
        .execute_task_plan(&task_plan)
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    Ok(Json(AutomationResponse {
        success: true,
        task_id: Uuid::new_v4().to_string(),
        results,
    }))
}

async fn extract_product_information(
    State(state): State<AppState>,
    Json(request): Json<ProductExtractionRequest>,
) -> Result<Json<ProductExtractionResponse>, AppError> {
    let start_time = std::time::Instant::now();

    info!(
        "Starting product information extraction for URL: {}",
        request.url
    );

    // Determine if we should use existing session or create a temporary one
    let use_existing_session = request.session_id.is_some();
    let session_id = request
        .session_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Get or create browser session
    let (html_content, _cleanup_session) = if use_existing_session {
        // Use existing session
        let mut sessions = state.browser_sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;

        // Navigate to URL and get page content
        session
            .navigate(&request.url)
            .await
            .map_err(|e| AppError::BrowserError(e.to_string()))?;

        // Wait a moment for page to load
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

        let content = session
            .interact(&BrowserAction::GetPageSource)
            .await
            .map_err(|e| AppError::BrowserError(e.to_string()))?;

        (content, false)
    } else {
        // Create temporary session
        let mut browser_session = browser::BrowserSession::new()
            .await
            .map_err(|e| AppError::BrowserError(e.to_string()))?;

        browser_session
            .navigate(&request.url)
            .await
            .map_err(|e| AppError::BrowserError(e.to_string()))?;

        // Wait a moment for page to load
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        let content = browser_session
            .interact(&BrowserAction::GetPageSource)
            .await
            .map_err(|e| AppError::BrowserError(e.to_string()))?;

        (content, true)
    };

    // Debug logging
    info!("HTML content length: {} characters", html_content.len());
    info!(
        "HTML content preview (first 500 chars): {}",
        &html_content[..std::cmp::min(500, html_content.len())]
    );

    // Extract product information using LLM
    let product_info = state
        .mcp_client
        .extract_product_information(&request.url, &html_content)
        .await
        .map_err(|e| AppError::MCPError(e.to_string()))?;

    let extraction_time = start_time.elapsed().as_millis() as u64;

    info!(
        "Product extraction completed in {}ms for URL: {}",
        extraction_time, request.url
    );
    info!("Extracted product info: {:?}", product_info);

    Ok(Json(ProductExtractionResponse {
        success: true,
        product: Some(product_info),
        error: None,
        extraction_time_ms: extraction_time,
    }))
}

// Debug endpoint to inspect raw page content
async fn debug_page_content(
    State(state): State<AppState>,
    Json(request): Json<types::DebugPageRequest>,
) -> Result<Json<types::DebugPageResponse>, AppError> {
    info!("Debug page content request for URL: {}", request.url);

    // Create temporary session for content retrieval
    let mut session = browser::BrowserSession::new()
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    session
        .navigate(&request.url)
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    // Wait for page to load
    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

    let page_content = session
        .interact(&BrowserAction::GetPageSource)
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    // Try to get page title via JavaScript
    let page_title = session
        .interact(&BrowserAction::ExecuteScript {
            script: "return document.title".to_string(),
        })
        .await
        .unwrap_or_else(|_| "Unknown".to_string());

    info!("Retrieved page content: {} characters", page_content.len());

    Ok(Json(types::DebugPageResponse {
        success: true,
        url: request.url,
        content_length: page_content.len(),
        content: page_content,
        title: page_title,
    }))
}
