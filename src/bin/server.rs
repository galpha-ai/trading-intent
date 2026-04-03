use tim::config::Config;
use tim::http::router;
use tim::schema::SchemaRegistry;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("tim=info".parse()?))
        .init();

    let config = Config::load()?;

    // Load intent schemas
    let registry = SchemaRegistry::load_from_dir(&config.intent_schemas)?;

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!(addr = %addr, "Starting TIM server");

    let app = router::build(config, registry);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
