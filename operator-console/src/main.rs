mod api;
mod config;
mod models;
mod views;

use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use api::{ApiClient, ApiClientError};
use axum::{
    Form, Json, Router,
    extract::{Request, State},
    http::{HeaderValue, header},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};
use config::Config;
use maud::Markup;
use models::{SemanticEvaluateForm, SemanticSearchForm, SourcePolicyForm};
use serde::Serialize;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    api: ApiClient,
}

#[derive(Serialize)]
struct ConsoleHealth<'a> {
    service: &'static str,
    status: &'static str,
    environment: &'a str,
    api_origin: String,
    production_ready: bool,
    crawler_controls_exposed: bool,
    notification_delivery_exposed: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Arc::new(Config::from_env()?);
    let api = ApiClient::new(&config).context("configure eal-api client")?;
    let state = AppState {
        config: Arc::clone(&config),
        api,
    };

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/healthz", get(healthz))
        .route("/sources", post(create_source))
        .route("/semantic/search", post(semantic_search))
        .route("/semantic/evaluate", post(semantic_evaluate))
        .layer(middleware::from_fn(security_headers))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let address = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .with_context(|| format!("bind operator console to {address}"))?;
    info!(address = %address, "Embedded Alerts operator console listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serve operator console")?;
    Ok(())
}

async fn dashboard(State(state): State<AppState>) -> Markup {
    let (health, sources) = tokio::join!(state.api.health(), state.api.list_sources());
    let health = health.map_err(|error| safe_api_error("load API health", &error));
    let sources = sources.map_err(|error| safe_api_error("load source policies", &error));
    views::dashboard(state.config.environment, &health, &sources)
}

async fn create_source(
    State(state): State<AppState>,
    Form(form): Form<SourcePolicyForm>,
) -> Markup {
    let policy = match form.into_policy() {
        Ok(policy) => policy,
        Err(error) => return views::operation_error("Source policy rejected", &error.to_string()),
    };
    match state.api.create_source(policy).await {
        Ok(source) => views::operation_result("Source policy registered", &source),
        Err(error) => api_operation_error("Source policy registration failed", &error),
    }
}

async fn semantic_search(
    State(state): State<AppState>,
    Form(form): Form<SemanticSearchForm>,
) -> Markup {
    let request = match form.into_payload() {
        Ok(request) => request,
        Err(error) => return views::operation_error("Semantic query rejected", &error.to_string()),
    };
    match state.api.semantic_search(&request).await {
        Ok(results) => views::operation_result("Semantic search results", &results),
        Err(error) => api_operation_error("Semantic search failed", &error),
    }
}

async fn semantic_evaluate(
    State(state): State<AppState>,
    Form(form): Form<SemanticEvaluateForm>,
) -> Markup {
    let request = match form.into_payload() {
        Ok(request) => request,
        Err(error) => {
            return views::operation_error("Candidate evaluation rejected", &error.to_string());
        }
    };
    match state.api.semantic_evaluate(&request).await {
        Ok(candidates) => views::operation_result("Match candidates created", &candidates),
        Err(error) => api_operation_error("Candidate evaluation failed", &error),
    }
}

async fn healthz(State(state): State<AppState>) -> Json<ConsoleHealth<'static>> {
    Json(ConsoleHealth {
        service: "eal-operator-console",
        status: "degraded",
        environment: state.config.environment.label(),
        api_origin: state.config.api_base_url.origin().ascii_serialization(),
        production_ready: false,
        crawler_controls_exposed: false,
        notification_delivery_exposed: false,
    })
}

fn api_operation_error(title: &str, error: &ApiClientError) -> Markup {
    warn!(status = ?error.status(), error = %error, operation = title, "operator API call failed");
    views::operation_error(title, &safe_api_error(title, error))
}

fn safe_api_error(operation: &str, error: &ApiClientError) -> String {
    let status = error
        .status()
        .map_or_else(|| "transport".to_owned(), |status| status.as_u16().to_string());
    format!("{operation}: API boundary returned {status}. {error}")
}

async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' https://unpkg.com; style-src 'self' 'unsafe-inline'; connect-src 'self'; img-src 'self' data:; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'",
        ),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    response
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if signal::ctrl_c().await.is_err() {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => warn!(%error, "failed to install SIGTERM handler"),
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
