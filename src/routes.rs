use axum::{
    Form, Json, Router,
    extract::{Path, State},
    response::Html,
    routing::{get, post},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    AppState,
    api::ApiClientError,
    models::{CreateSourceDomain, SearchForm, SemanticSearchRequest, SourceForm},
    views::{self, Notice, NoticeKind},
};

pub(crate) fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/healthz", get(health))
        .route("/partials/overview", get(overview))
        .route("/partials/sources", get(sources).post(create_source))
        .route("/partials/sources/{source_id}/scan", post(scan_source))
        .route("/partials/pages", get(pages))
        .route("/partials/search", post(search))
        .route("/partials/matches", get(matches))
        .with_state(state)
}

async fn index(State(state): State<AppState>) -> Html<String> {
    Html(
        views::layout(state.environment, state.tenant_id)
            .into_string(),
    )
}

#[derive(Debug, Serialize)]
struct ConsoleHealth {
    service: &'static str,
    status: &'static str,
    environment: &'static str,
    production_ready: bool,
    tenant_context: &'static str,
    api_reachable: bool,
    api_production_ready: Option<bool>,
}

async fn health(State(state): State<AppState>) -> Json<ConsoleHealth> {
    let api_health = state.api.health().await.ok();
    Json(ConsoleHealth {
        service: "eal-mash-web",
        status: if api_health.is_some() {
            "degraded"
        } else {
            "unavailable"
        },
        environment: state.environment.as_str(),
        production_ready: false,
        tenant_context: "development_header",
        api_reachable: api_health.is_some(),
        api_production_ready: api_health.map(|health| health.production_ready),
    })
}

async fn overview(State(state): State<AppState>) -> Html<String> {
    let markup = match state.api.health().await {
        Ok(health) => views::overview_panel(&health),
        Err(error) => views::notice(&api_notice(error)),
    };
    Html(markup.into_string())
}

async fn sources(State(state): State<AppState>) -> Html<String> {
    Html(render_sources(&state, None).await.into_string())
}

async fn create_source(
    State(state): State<AppState>,
    Form(form): Form<SourceForm>,
) -> Html<String> {
    let source = match CreateSourceDomain::try_from(form) {
        Ok(source) => source,
        Err(message) => {
            return Html(
                render_sources(
                    &state,
                    Some(Notice::new(
                        NoticeKind::Error,
                        "Source policy rejected",
                        message,
                    )),
                )
                .await
                .into_string(),
            );
        }
    };

    let notice = match state.api.create_source(&source).await {
        Ok(created) => Notice::new(
            NoticeKind::Success,
            "Source registered",
            format!(
                "{} is registered for bounded public-page discovery. No notification was sent.",
                created.host
            ),
        ),
        Err(error) => api_notice(error),
    };
    Html(
        render_sources(&state, Some(notice))
            .await
            .into_string(),
    )
}

async fn render_sources(state: &AppState, notice: Option<Notice>) -> maud::Markup {
    match state.api.list_sources().await {
        Ok(sources) => {
            let notice = notice.or_else(|| {
                sources.is_empty().then(|| {
                    Notice::new(
                        NoticeKind::Info,
                        "Index boundary not established",
                        "Register an exact public domain before any page can be fetched or matched.",
                    )
                })
            });
            views::sources_panel(&sources, notice.as_ref())
        }
        Err(error) => {
            let fallback = notice.unwrap_or_else(|| api_notice(error));
            views::sources_panel(&[], Some(&fallback))
        }
    }
}

async fn scan_source(
    State(state): State<AppState>,
    Path(source_id): Path<Uuid>,
) -> Html<String> {
    let markup = match state.api.scan_source(source_id).await {
        Ok(report) => views::scan_report(&report),
        Err(error) => views::notice(&api_notice(error)),
    };
    Html(markup.into_string())
}

async fn pages(State(state): State<AppState>) -> Html<String> {
    let markup = match state.api.list_pages().await {
        Ok(pages) => views::pages_panel(&pages),
        Err(error) => views::notice(&api_notice(error)),
    };
    Html(markup.into_string())
}

async fn search(
    State(state): State<AppState>,
    Form(form): Form<SearchForm>,
) -> Html<String> {
    let request = match SemanticSearchRequest::try_from(form) {
        Ok(request) => request,
        Err(message) => {
            return Html(
                views::notice(&Notice::new(
                    NoticeKind::Error,
                    "Search rejected",
                    message,
                ))
                .into_string(),
            );
        }
    };

    let markup = match state.api.search(&request).await {
        Ok(response) => views::search_results(&response),
        Err(error) => views::notice(&api_notice(error)),
    };
    Html(markup.into_string())
}

async fn matches(State(state): State<AppState>) -> Html<String> {
    let markup = match state.api.list_matches().await {
        Ok(matches) => views::matches_panel(&matches),
        Err(error) => views::notice(&api_notice(error)),
    };
    Html(markup.into_string())
}

fn api_notice(error: ApiClientError) -> Notice {
    Notice::new(
        NoticeKind::Error,
        format!("API error · {}", error.code),
        error.message,
    )
}
