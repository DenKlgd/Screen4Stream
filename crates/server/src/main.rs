//! Запуск без окна Tauri: интерфейс открывается в обычном браузере.
//!
//!     cargo run -p s4s-server -- --open

use s4s_server::{Config, Server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    s4s_server::init_logging();
    s4s_core::init()?;

    let server = Server::bind(Config::default()).await?;
    let url = server.local_url();
    println!("Screen4Stream запущен: {url}");
    if std::env::args().any(|arg| arg == "--open") {
        s4s_server::open_in_browser(&url);
    }
    server.run().await
}
