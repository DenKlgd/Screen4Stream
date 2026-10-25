//! Состояние трансляции: что сейчас в эфире и с какими настройками.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use s4s_core::{Source, SourceRef, Stream, StreamEvent, StreamSettings};
use serde::Serialize;

use crate::Config;
use crate::net;

/// Сколько источник может молчать, прежде чем мы предупредим ведущего.
const NO_VIDEO_GRACE: Duration = Duration::from_secs(2);

pub struct AppState {
    pub config: Config,
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    live: Option<Live>,
    /// Растёт при каждом запуске: события от старого пайплайна
    /// не должны останавливать новый (например, после смены источника).
    generation: u64,
    last_error: Option<String>,
}

struct Live {
    stream: Stream,
    source: Source,
    settings: StreamSettings,
    started: Instant,
}

/// То, что видит интерфейс ведущего.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub live: bool,
    pub source: Option<Source>,
    pub settings: Option<StreamSettings>,
    pub elapsed_secs: u64,
    pub viewers: usize,
    /// Источник уже пару секунд не присылает кадров.
    pub no_video: bool,
    pub share_url: String,
    pub last_error: Option<String>,
}

/// То, что видит зритель: без настроек и прочих деталей ведущего.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicStatus {
    pub live: bool,
    pub title: Option<String>,
    pub signalling_port: u16,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            inner: Mutex::default(),
        }
    }

    fn signalling_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.config.signalling_port)
    }

    /// Запускает трансляцию. Если что-то уже в эфире — это смена источника.
    pub fn start(self: &Arc<Self>, source_id: SourceRef, settings: StreamSettings) -> Result<Status> {
        let source = find_source(source_id).context("источник не найден — возможно, окно закрыли")?;

        let mut inner = self.inner.lock().unwrap();
        // Старый пайплайн останавливаем до запуска нового: оба захватывали бы один экран.
        drop(inner.live.take());
        inner.generation += 1;
        inner.last_error = None;

        let generation = inner.generation;
        let state = Arc::clone(self);
        let stream = Stream::start(source_id, &settings, &self.signalling_url(), move |event| {
            state.on_stream_event(generation, event)
        })?;

        inner.live = Some(Live {
            stream,
            source,
            settings,
            started: Instant::now(),
        });
        Ok(self.status_locked(&inner))
    }

    pub fn stop(&self) -> Status {
        let mut inner = self.inner.lock().unwrap();
        let live = inner.live.take();
        inner.last_error = None;
        let status = self.status_locked(&inner);
        drop(inner);
        drop(live);
        status
    }

    pub fn status(&self) -> Status {
        self.status_locked(&self.inner.lock().unwrap())
    }

    pub fn public_status(&self) -> PublicStatus {
        let inner = self.inner.lock().unwrap();
        PublicStatus {
            live: inner.live.is_some(),
            title: inner.live.as_ref().map(|live| live.source.title.clone()),
            signalling_port: self.config.signalling_port,
        }
    }

    fn status_locked(&self, inner: &Inner) -> Status {
        let host = net::lan_ip().map_or_else(|| "127.0.0.1".to_owned(), |ip| ip.to_string());
        Status {
            live: inner.live.is_some(),
            source: inner.live.as_ref().map(|live| live.source.clone()),
            settings: inner.live.as_ref().map(|live| live.settings.clone()),
            elapsed_secs: inner.live.as_ref().map_or(0, |live| live.started.elapsed().as_secs()),
            viewers: inner.live.as_ref().map_or(0, |live| live.stream.viewer_count()),
            no_video: inner
                .live
                .as_ref()
                .is_some_and(|live| live.stream.since_last_frame() > NO_VIDEO_GRACE),
            share_url: format!("http://{host}:{}/watch", self.config.http_port),
            last_error: inner.last_error.clone(),
        }
    }

    /// Вызывается из потока GStreamer, когда трансляция закончилась сама.
    fn on_stream_event(&self, generation: u64, event: StreamEvent) {
        let mut inner = self.inner.lock().unwrap();
        if inner.generation != generation {
            return;
        }
        let live = inner.live.take();
        inner.last_error = Some(match event {
            StreamEvent::Ended => "Источник закрыт — трансляция остановлена".to_owned(),
            StreamEvent::Error(message) => format!("Трансляция прервана: {message}"),
        });
        drop(inner);
        drop(live);
    }
}

fn find_source(id: SourceRef) -> Option<Source> {
    let id = id.to_string();
    s4s_core::sources::monitors()
        .into_iter()
        .chain(s4s_core::sources::windows())
        .find(|source| source.id == id)
}
