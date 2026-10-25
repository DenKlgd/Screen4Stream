// Последние выбранные настройки трансляции — запоминаются в браузере.

const KEY = 'screen4stream.settings';

const DEFAULTS = { height: 1080, fps: 30, cursor: true, audio: true };

export function loadSettings() {
  try {
    return { ...DEFAULTS, ...JSON.parse(localStorage.getItem(KEY) ?? '{}') };
  } catch {
    return { ...DEFAULTS };
  }
}

export function saveSettings(settings) {
  try {
    localStorage.setItem(KEY, JSON.stringify(settings));
  } catch {
    // Хранилище недоступно (например, приватный режим) — не страшно.
  }
}
