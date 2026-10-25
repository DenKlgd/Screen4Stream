//! Веб-интерфейс, вшитый в бинарник.
//!
//! В release-сборке файлы из `web/dist` попадают внутрь exe.
//! В debug-сборке читаются с диска — после `npm run build` достаточно обновить страницу.

use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../web/dist"]
#[allow_missing = true]
struct Assets;

pub async fn serve(uri: Uri) -> Response {
    let path = match uri.path().trim_matches('/') {
        "" => "index.html",
        "watch" => "watch.html",
        other => other,
    };

    match Assets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], file.data).into_response()
        }
        None if Assets::get("index.html").is_none() => (
            StatusCode::NOT_FOUND,
            "Веб-интерфейс не собран: выполните `npm run build` в папке проекта",
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
