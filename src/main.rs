mod api;
mod config;
mod models;
mod routes;
mod views;

use std::sync::Arc;

use anyhow::Context;
use api::ApiClient;
use axum::Router;
use config::{AppEnvironment, ConsoleConfig};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) api: Arc<ApiClient>,
    pub(crate) environment: AppEnvironment,
    pub(crate) tenant_id: Uuid,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = ConsoleConfig::from_env()?;
    let api = ApiClient::new(config.api_base_url.clone(), config.tenant_id)
        .context("configure Embedded Alerts API client")?;
    let state = AppState {
        api: Arc::new(api),
        environment: config.environment,
        tenant_id: config.tenant_id,
    };

    warn!(
        environment = state.environment.as_str(),
        tenant_context = "development_header",
        api_base_url = %config.api_base_url,
        "Mash console is a development/operator surface; production startup is disabled"
    );

    let app: Router = routes::router(state).layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind((config.host.as_str(), config.port))
        .await
        .with_context(|| format!("bind {}:{}", config.host, config.port))?;
    info!(address = %listener.local_addr()?, "Embedded Alerts Mash console listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
