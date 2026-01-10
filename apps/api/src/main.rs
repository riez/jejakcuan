use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use jejakcuan_api::{config::Config, create_app};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "jejakcuan_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = Config::from_env();
    tracing::info!("Starting JejakCuan API on {}:{}", config.host, config.port);

    // Connect to database
    let db = jejakcuan_db::create_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Connected to database");

    // Build the application
    let app = create_app(db, config.clone());

    // Run server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
