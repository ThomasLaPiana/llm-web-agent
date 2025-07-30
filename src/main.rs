use llm_web_agent::{create_router, AppState};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("Starting LLM Web Agent with Llama + MCP support...");

    // Create application state
    let state = AppState::new().await?;

    // Build our application with routes
    let app = create_router()
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Determine port
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    // Run the server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Server running on http://0.0.0.0:{}", port);
    info!(
        "MCP manifest available at: http://0.0.0.0:{}/.well-known/mcp/manifest.json",
        port
    );
    info!("API endpoints available at: http://0.0.0.0:{}/api/", port);

    axum::serve(listener, app).await?;

    Ok(())
}
