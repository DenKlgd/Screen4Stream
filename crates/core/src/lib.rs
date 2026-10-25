//! Ядро Screen4Stream: список источников (окна и мониторы), их миниатюры
//! и WebRTC-пайплайн на GStreamer.

pub mod sources;
pub mod stream;
pub mod thumbnail;

pub use sources::{Source, SourceKind, SourceRef};
pub use stream::{Stream, StreamEvent, StreamSettings};

use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};

/// Вызывается один раз при старте приложения.
pub fn init() -> anyhow::Result<()> {
    // Без этого Windows отдаёт размеры окон «в логических пикселях»,
    // и на мониторах с масштабом 125–150% миниатюры обрезаются.
    // Ошибка значит, что режим уже выставлен (например, Tauri), это нормально.
    let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    gst::init()?;
    Ok(())
}
