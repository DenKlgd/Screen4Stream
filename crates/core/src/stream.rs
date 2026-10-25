//! WebRTC-трансляция: захват → GPU-преобразование → webrtcsink.
//!
//! `webrtcsink` сам выбирает кодировщик (NVENC, Media Foundation, x264, VP8…),
//! подстраивает битрейт под сеть каждого зрителя и общается со зрителями
//! через signaling-сервер по WebSocket.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use gst::prelude::*;
use serde::{Deserialize, Serialize};

use crate::SourceRef;

/// Настройки, которые пользователь выбирает перед выходом в эфир.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamSettings {
    /// Высота кадра в пикселях (ширина подбирается с сохранением пропорций).
    /// `None` — исходный размер.
    pub height: Option<u32>,
    pub fps: u32,
    pub cursor: bool,
    /// Транслировать системный звук (то, что слышно в колонках).
    pub audio: bool,
}

/// Что случилось с трансляцией без участия пользователя.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Источник закончился (например, окно закрыли).
    Ended,
    Error(String),
}

/// Работающая трансляция. Остановка — просто `drop`.
pub struct Stream {
    pipeline: gst::Pipeline,
    sink: gst::Element,
    frames: FrameWatch,
}

impl Stream {
    /// Запускает пайплайн. `on_event` вызывается из отдельного потока,
    /// когда трансляция завершилась сама или с ошибкой.
    pub fn start(
        source: SourceRef,
        settings: &StreamSettings,
        signalling_url: &str,
        on_event: impl Fn(StreamEvent) + Send + 'static,
    ) -> Result<Self> {
        let description = pipeline_description(source, settings, signalling_url);
        tracing::info!(%description, "запуск пайплайна");

        let pipeline = gst::parse::launch(&description)
            .context("не удалось собрать пайплайн")?
            .downcast::<gst::Pipeline>()
            .map_err(|_| anyhow!("ожидался gst::Pipeline"))?;
        let sink = pipeline
            .by_name("webrtc")
            .context("в пайплайне нет webrtcsink")?;
        let frames = FrameWatch::attach(&pipeline)?;

        watch_bus(&pipeline, on_event);
        pipeline
            .set_state(gst::State::Playing)
            .context("не удалось запустить пайплайн")?;
        Ok(Self {
            pipeline,
            sink,
            frames,
        })
    }

    /// Сколько зрителей подключено: в статистике webrtcsink по полю на каждого.
    pub fn viewer_count(&self) -> usize {
        self.sink.property::<gst::Structure>("stats").n_fields()
    }

    /// Сколько времени источник не присылал кадров (или с запуска, если не прислал ни одного).
    ///
    /// Окно отдаёт кадры, только когда перерисовывается. Некоторые программы
    /// (например, Windows Terminal) не рисуют себя, пока перекрыты другими окнами:
    /// приходит один кадр, и всё — кодировщику этого мало, зрители ничего не видят.
    pub fn since_last_frame(&self) -> Duration {
        self.frames.since_last_frame()
    }
}

/// «Датчик» на выходе источника захвата: запоминает время последнего кадра.
struct FrameWatch {
    started: Instant,
    /// Миллисекунды от `started` до последнего кадра; `u64::MAX` — кадров ещё не было.
    last_frame_ms: Arc<AtomicU64>,
}

impl FrameWatch {
    fn attach(pipeline: &gst::Pipeline) -> Result<Self> {
        let started = Instant::now();
        let last_frame_ms = Arc::new(AtomicU64::new(u64::MAX));
        let last = Arc::clone(&last_frame_ms);
        pipeline
            .by_name("capture")
            .and_then(|capture| capture.static_pad("src"))
            .context("в пайплайне нет источника захвата")?
            .add_probe(gst::PadProbeType::BUFFER, move |_, _| {
                last.store(started.elapsed().as_millis() as u64, Ordering::Relaxed);
                gst::PadProbeReturn::Ok
            });
        Ok(Self {
            started,
            last_frame_ms,
        })
    }

    fn since_last_frame(&self) -> Duration {
        let elapsed = self.started.elapsed();
        match self.last_frame_ms.load(Ordering::Relaxed) {
            u64::MAX => elapsed,
            ms => elapsed.saturating_sub(Duration::from_millis(ms)),
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

fn pipeline_description(
    source: SourceRef,
    settings: &StreamSettings,
    signalling_url: &str,
) -> String {
    let capture = match source {
        // Захват отдельного окна возможен только через Windows Graphics Capture.
        SourceRef::Window(hwnd) => {
            format!("d3d11screencapturesrc name=capture capture-api=wgc window-handle={hwnd}")
        }
        SourceRef::Monitor(hmon) => {
            format!("d3d11screencapturesrc name=capture monitor-handle={hmon}")
        }
    };
    let scale = settings
        .height
        .map(|h| format!(",height={h}"))
        .unwrap_or_default();

    // Масштабирование и перевод в NV12 делаем на видеокарте (d3d11convert),
    // в обычную память забираем уже готовый кадр — так дешевле для CPU.
    // pixel-aspect-ratio=1/1 обязателен: без него d3d11convert оставляет ширину
    // как есть и меняет пропорции пикселя, а браузер их игнорирует — картинка сплющится.
    let mut description = format!(
        "{capture} show-cursor={cursor} \
         ! video/x-raw(memory:D3D11Memory),framerate={fps}/1 \
         ! d3d11convert \
         ! video/x-raw(memory:D3D11Memory),format=NV12,pixel-aspect-ratio=1/1{scale} \
         ! d3d11download ! video/x-raw,format=NV12 ! queue \
         ! webrtcsink name=webrtc video-caps=\"video/x-h264;video/x-vp8\" \
           signaller::uri={signalling_url}",
        cursor = settings.cursor,
        fps = settings.fps,
    );
    if settings.audio {
        description.push_str(
            " wasapi2src loopback=true low-latency=true \
             ! audioconvert ! audioresample ! queue ! webrtc.",
        );
    }
    description
}

/// Следит за шиной пайплайна в отдельном потоке, пока пайплайн жив.
fn watch_bus(pipeline: &gst::Pipeline, on_event: impl Fn(StreamEvent) + Send + 'static) {
    let bus = pipeline.bus().expect("у пайплайна всегда есть шина");
    let pipeline = pipeline.downgrade();

    thread::Builder::new()
        .name("gst-bus".into())
        .spawn(move || {
            while pipeline.upgrade().is_some() {
                let Some(message) = bus.timed_pop(gst::ClockTime::from_mseconds(200)) else {
                    continue;
                };
                match message.view() {
                    gst::MessageView::Eos(_) => {
                        on_event(StreamEvent::Ended);
                        break;
                    }
                    gst::MessageView::Error(err) => {
                        tracing::error!(debug = ?err.debug(), "ошибка GStreamer: {}", err.error());
                        on_event(StreamEvent::Error(err.error().to_string()));
                        break;
                    }
                    _ => {}
                }
            }
        })
        .expect("не удалось запустить поток шины GStreamer");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(height: Option<u32>, audio: bool) -> StreamSettings {
        StreamSettings { height, fps: 30, cursor: true, audio }
    }

    #[test]
    fn window_uses_wgc_and_scales() {
        let d = pipeline_description(
            SourceRef::Window(0x1234),
            &settings(Some(720), false),
            "ws://127.0.0.1:8443",
        );
        assert!(d.contains("capture-api=wgc window-handle=4660"));
        assert!(d.contains("pixel-aspect-ratio=1/1,height=720"));
        assert!(d.contains("framerate=30/1"));
        assert!(!d.contains("wasapi2src"));
    }

    #[test]
    fn monitor_with_audio_and_source_size() {
        let d = pipeline_description(
            SourceRef::Monitor(7),
            &settings(None, true),
            "ws://127.0.0.1:8443",
        );
        assert!(d.contains("monitor-handle=7"));
        assert!(d.contains("pixel-aspect-ratio=1/1 !"));
        assert!(d.contains("wasapi2src loopback=true"));
    }
}
