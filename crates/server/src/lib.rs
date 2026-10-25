//! Сервер Screen4Stream.
//!
//! Один процесс поднимает два сервера:
//! - HTTP (по умолчанию порт 8080): веб-интерфейс, страница зрителя и API;
//! - WebRTC signaling (порт 8443): через него webrtcsink и браузеры зрителей
//!   договариваются о соединении. Само видео идёт напрямую, мимо этих серверов.

mod api;
mod assets;
mod net;
mod signalling;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::net::TcpListener;

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct Config {
    pub http_port: u16,
    pub signalling_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http_port: 8080,
            signalling_port: 8443,
        }
    }
}

/// Сервер с уже занятыми портами. Разделение на `bind` и `run` нужно,
/// чтобы окно Tauri открывалось только когда сервер точно готов.
pub struct Server {
    http: TcpListener,
    signalling: TcpListener,
    state: Arc<AppState>,
}

impl Server {
    pub async fn bind(config: Config) -> Result<Self> {
        let http = TcpListener::bind(("0.0.0.0", config.http_port))
            .await
            .with_context(|| format!("не удалось занять порт {}", config.http_port))?;
        let signalling = TcpListener::bind(("0.0.0.0", config.signalling_port))
            .await
            .with_context(|| format!("не удалось занять порт {}", config.signalling_port))?;
        Ok(Self {
            http,
            signalling,
            state: Arc::new(AppState::new(config)),
        })
    }

    /// Адрес интерфейса для этого компьютера.
    pub fn local_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.state.config.http_port)
    }

    pub async fn run(self) -> Result<()> {
        let app = api::router(self.state)
            .into_make_service_with_connect_info::<SocketAddr>();
        tokio::try_join!(
            async { axum::serve(self.http, app).await.context("HTTP-сервер упал") },
            signalling::serve(self.signalling),
        )?;
        Ok(())
    }
}

/// Открывает адрес в браузере по умолчанию.
pub fn open_in_browser(url: &str) {
    if let Err(err) = std::process::Command::new("explorer").arg(url).spawn() {
        tracing::warn!("не удалось открыть браузер: {err}");
    }
}

/// Логи в консоль; уровень меняется переменной `RUST_LOG`, например `RUST_LOG=debug`.
pub fn init_logging() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
