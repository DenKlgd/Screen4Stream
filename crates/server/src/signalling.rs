//! WebRTC signaling-сервер из gst-plugins-rs, запущенный внутри нашего процесса.
//!
//! Протокол — JSON по WebSocket. Его же реализует `web/src/lib/signalling.js`
//! на стороне браузера зрителя.

use anyhow::Result;
use gst_plugin_webrtc_signalling::handlers::Handler;
use gst_plugin_webrtc_signalling::server::Server;
use tokio::net::TcpListener;

pub async fn serve(listener: TcpListener) -> Result<()> {
    let server = Server::spawn(Handler::new);
    loop {
        let (stream, address) = listener.accept().await?;
        tracing::debug!(%address, "signaling: новое подключение");
        let mut server = server.clone();
        tokio::spawn(async move {
            if let Err(err) = server.accept_async(stream).await {
                tracing::warn!(%address, "signaling: {err}");
            }
        });
    }
}
