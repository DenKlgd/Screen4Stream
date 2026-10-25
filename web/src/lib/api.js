// Обёртка над HTTP API Rust-сервера (crates/server/src/api.rs).

async function request(method, url, body) {
  const response = await fetch(url, {
    method,
    headers: body ? { 'Content-Type': 'application/json' } : {},
    body: body ? JSON.stringify(body) : undefined,
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(data.error ?? `Ошибка сервера: ${response.status}`);
  }
  return data;
}

/** Состояние трансляции для ведущего. */
export const getStatus = () => request('GET', '/api/status');

/** Списки `{ monitors, windows }`. */
export const getSources = () => request('GET', '/api/sources');

/** Адрес свежей миниатюры (параметр t не даёт браузеру взять старую из кэша). */
export const thumbnailUrl = (sourceId, width = 480) =>
  `/api/sources/${sourceId}/thumbnail?w=${width}&t=${Date.now()}`;

/** settings: `{ sourceId, height, fps, cursor, audio }` */
export const startStream = (settings) => request('POST', '/api/stream/start', settings);

export const stopStream = () => request('POST', '/api/stream/stop');

/** Просит сервер открыть интерфейс в браузере по умолчанию (нужно внутри окна Tauri). */
export const openInBrowser = () => request('POST', '/api/open-in-browser');

/** Что видит зритель: `{ live, title, signallingPort }`. */
export const getPublicStream = () => request('GET', '/api/public/stream');

/** 3725 → "01:02:05" */
export function formatDuration(totalSeconds) {
  const pad = (n) => String(n).padStart(2, '0');
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor(totalSeconds / 60) % 60;
  return `${pad(hours)}:${pad(minutes)}:${pad(totalSeconds % 60)}`;
}
