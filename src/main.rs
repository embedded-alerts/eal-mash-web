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
struct AppState {
    // Scaffold readiness signal only: this is not an approved P1 reader. Production product
    // reads/writes must follow the tenant-safe P1/P2 boundary in docs/web-api-data-access.md.
    db: Option<DatabaseConnection>,
    items: Arc<RwLock<Vec<Item>>>,
    events: broadcast::Sender<String>,
    supabase_url: Option<String>,
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

async fn index(State(state): State<AppState>) -> Html<String> {
    let items = state.items.read().await.clone();
    Html(layout(items_markup(&items)).into_string())
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    axum::Json(serde_json::json!({"status":"ok","database":state.db.is_some(),"supabase":state.supabase_url.is_some()}))
}

async fn items_partial(State(state): State<AppState>) -> Html<String> {
    Html(items_markup(&state.items.read().await).into_string())
}

// Scaffold-only in-memory mutation, not authoritative persistence. A production implementation
// sends the bounded, authenticated, idempotent command to eal-api over P2.
async fn create_item(State(state): State<AppState>, Form(input): Form<NewItem>) -> Html<String> {
    let item = Item { id: Uuid::new_v4(), title: input.title, detail: input.detail };
    state.items.write().await.push(item.clone());
    let _ = state.events.send(format!("created:{}", item.id));
    Html(items_markup(&state.items.read().await).into_string())
}

fn layout(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Embedded Alerts" }
                script src="https://unpkg.com/htmx.org@2.0.4" {}
                style { "body{font-family:system-ui;max-width:900px;margin:3rem auto;padding:0 1rem} form{display:grid;gap:.75rem} .card{border:1px solid #ddd;border-radius:12px;padding:1rem;margin:.75rem 0}" }
            }
            body {
                header { h1 { "Embedded Alerts" } p { "Embedding-native monitoring and alerting that continuously matches user intents against newly ingested documents, feeds, pages, and streams." } }
                form hx-post="/partials/alerts" hx-target="#items" hx-swap="innerHTML" {
                    input type="text" name="title" placeholder="Title" required;
                    textarea name="detail" placeholder="Details" required {}
                    button type="submit" { "Create AlertRule" }
                }
                section id="items" { (content) }
                script { (maud::PreEscaped("const proto=location.protocol==='https:'?'wss':'ws';const ws=new WebSocket(proto+'://'+location.host+'/ws');ws.onmessage=()=>htmx.ajax('GET','/partials/alerts',{target:'#items'});")) }
            }
        }
    }
}

fn items_markup(items: &[Item]) -> Markup {
    html! { @for item in items { article class="card" data-id=(item.id) { h2 { (item.title) } p { (item.detail) } } } }
}

fn seed_items() -> Vec<Item> { vec![Item { id: Uuid::new_v4(), title: "Foundation ready".into(), detail: "Maud + Axum + SeaORM + Supabase configuration + HTMX + WebSockets".into() }] }

async fn ws_upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse { ws.on_upgrade(move |socket| websocket(socket, state)) }
async fn websocket(socket: WebSocket, state: AppState) {
    let (mut tx, mut rx) = socket.split();
    let mut events = state.events.subscribe();
    loop {
        tokio::select! {
            message = rx.next() => match message { Some(Ok(Message::Close(_))) | None => break, _ => {} },
            event = events.recv() => match event { Ok(event) => if tx.send(Message::Text(event.into())).await.is_err() { break; }, Err(_) => break },
        }
    }
}
