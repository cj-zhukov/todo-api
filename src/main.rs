use todo::{config::Config, create_router, init_tracing};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let config = Config::new("config.json").await?;
    let pool = config.connect().await?;
    let router = create_router(pool).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, router).await?;

    Ok(())
}
