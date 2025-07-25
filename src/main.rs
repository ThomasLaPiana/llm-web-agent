#[tokio::main]
async fn main() -> anyhow::Result<()> {
    llm_web_agent::run_server().await
}
