use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
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
        .route("/browser/navigate", post(navigate_to_url))
        .route("/browser/interact", post(interact_with_page))
        .route("/browser/extract", post(extract_page_data))
        .route("/automation/task", post(execute_automation_task))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "LLM Web Agent is running!"
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
