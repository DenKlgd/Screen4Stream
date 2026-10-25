//! HTTP API.
//!
//! - `/api/public/*` — для зрителей, доступно из локальной сети;
//! - остальное `/api/*` — управление трансляцией, только с этого компьютера.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{ConnectInfo, Path, Query, Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use s4s_core::{Source, SourceRef, StreamSettings};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::assets;
use crate::state::{AppState, PublicStatus, Status};

type AppStateRef = Arc<AppState>;

pub fn router(state: AppStateRef) -> Router {
    let control = Router::new()
        .route("/status", get(status))
        .route("/sources", get(sources))
        .route("/sources/{id}/thumbnail", get(thumbnail))
        .route("/stream/start", post(start))
        .route("/stream/stop", post(stop))
        .route("/open-in-browser", post(open_in_browser))
        .route_layer(middleware::from_fn(local_only));

    let public = Router::new().route("/stream", get(public_status));

    Router::new()
        .nest("/api", control.nest("/public", public))
        .fallback(assets::serve)
        .with_state(state)
}

/// Пускает к управлению только запросы с этого же компьютера.
///
/// Проверяем и адрес клиента, и заголовок Host: второе защищает
/// от DNS rebinding, когда чужой сайт пытается обратиться к localhost.
async fn local_only(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    let host_is_local = host.starts_with("127.0.0.1") || host.starts_with("localhost");
    if addr.ip().is_loopback() && host_is_local {
        next.run(request).await
    } else {
        ApiError(StatusCode::FORBIDDEN, "Управление доступно только с этого компьютера".into())
            .into_response()
    }
}

async fn status(State(state): State<AppStateRef>) -> Json<Status> {
    Json(state.status())
}

async fn public_status(State(state): State<AppStateRef>) -> Json<PublicStatus> {
    Json(state.public_status())
}

#[derive(Serialize)]
struct SourceList {
    monitors: Vec<Source>,
    windows: Vec<Source>,
}

async fn sources() -> Result<Json<SourceList>, ApiError> {
    let list = tokio::task::spawn_blocking(|| SourceList {
        monitors: s4s_core::sources::monitors(),
        windows: s4s_core::sources::windows(),
    })
    .await
    .map_err(anyhow::Error::from)?;
    Ok(Json(list))
}

#[derive(Deserialize)]
struct ThumbnailQuery {
    /// Максимальная ширина миниатюры в пикселях.
    w: Option<u32>,
}

async fn thumbnail(
    Path(id): Path<String>,
    Query(query): Query<ThumbnailQuery>,
) -> Result<Response, ApiError> {
    let source = parse_source(&id)?;
    let width = query.w.unwrap_or(480).clamp(64, 1920);
    let jpeg = tokio::task::spawn_blocking(move || s4s_core::thumbnail::capture_jpeg(source, width))
        .await
        .map_err(anyhow::Error::from)??;
    Ok((
        [
            (header::CONTENT_TYPE, "image/jpeg"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        jpeg,
    )
        .into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartRequest {
    source_id: String,
    #[serde(flatten)]
    settings: StreamSettings,
}

async fn start(
    State(state): State<AppStateRef>,
    Json(request): Json<StartRequest>,
) -> Result<Json<Status>, ApiError> {
    let source = parse_source(&request.source_id)?;
    let status = tokio::task::spawn_blocking(move || state.start(source, request.settings))
        .await
        .map_err(anyhow::Error::from)??;
    Ok(Json(status))
}

async fn stop(State(state): State<AppStateRef>) -> Json<Status> {
    Json(state.stop())
}

/// Кнопка «Открыть в браузере» в окне Tauri: из WebView самому браузер не открыть.
async fn open_in_browser(State(state): State<AppStateRef>) -> StatusCode {
    crate::open_in_browser(&format!("http://127.0.0.1:{}", state.config.http_port));
    StatusCode::NO_CONTENT
}

fn parse_source(id: &str) -> Result<SourceRef, ApiError> {
    id.parse()
        .map_err(|err: anyhow::Error| ApiError(StatusCode::BAD_REQUEST, err.to_string()))
}

/// Ошибка, которую фронтенд получает как `{ "error": "текст" }`.
struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        tracing::warn!("{err:#}");
        Self(StatusCode::INTERNAL_SERVER_ERROR, format!("{err:#}"))
    }
}
