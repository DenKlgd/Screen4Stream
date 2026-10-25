<!--
  Страница зрителя: открывается по ссылке /watch на любом устройстве в сети.
-->
<script>
  import { onMount } from 'svelte';
  import Icon from '../lib/Icon.svelte';
  import { getPublicStream } from '../lib/api.js';
  import { connectViewer } from '../lib/signalling.js';

  let phase = $state('connecting');
  let title = $state('');
  let muted = $state(true);
  let showStats = $state(false);
  let stats = $state(null);
  let video;
  let player;
  let viewer = null;

  const messages = {
    connecting: ['Подключение к трансляции…', 'Устанавливаем WebRTC-соединение с компьютером ведущего'],
    waiting: ['Трансляция ещё не началась', 'Страница подключится сама, как только ведущий выйдет в эфир'],
    ended: ['Трансляция остановлена', 'Если ведущий начнёт снова, страница подключится сама'],
    offline: ['Нет связи с компьютером ведущего', 'Пробуем переподключиться…'],
    failed: [
      'Не удалось установить соединение',
      'Возможно, брандмауэр Windows на компьютере ведущего блокирует входящие подключения',
    ],
  };

  const badge = $derived(
    phase === 'live' ? { text: 'В ЭФИРЕ', live: true }
    : phase === 'ended' ? { text: 'ЗАВЕРШЕНО', live: false }
    : { text: 'ОЖИДАНИЕ', live: false },
  );

  async function updateTitle() {
    try {
      title = (await getPublicStream()).title ?? '';
    } catch {
      // Заголовок не критичен.
    }
  }

  onMount(() => {
    let cancelled = false;

    getPublicStream()
      .then((info) => {
        if (cancelled) return;
        title = info.title ?? '';
        viewer = connectViewer({
          url: `ws://${location.hostname}:${info.signallingPort}`,
          onStream: (stream) => {
            video.srcObject = stream;
          },
          onState: (state) => {
            phase = state;
            if (state === 'live') updateTitle();
          },
        });
      })
      .catch(() => (phase = 'offline'));

    // Статистику считаем раз в секунду; битрейт — по приросту полученных байт.
    let previous = null;
    const timer = setInterval(async () => {
      const current = await viewer?.getStats();
      if (current && previous && current.timestamp > previous.timestamp) {
        const bits = (current.bytesReceived - previous.bytesReceived) * 8;
        current.bitrateMbps = bits / ((current.timestamp - previous.timestamp) / 1000) / 1e6;
      }
      previous = current;
      stats = current;
    }, 1000);

    return () => {
      cancelled = true;
      clearInterval(timer);
      viewer?.close();
    };
  });

  function toggleMute() {
    muted = !muted;
    video.muted = muted;
    if (!muted) video.play();
  }

  function toggleFullscreen() {
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      player.requestFullscreen();
    }
  }

  const format = (value, digits = 0) => (value == null ? '—' : value.toFixed(digits).replace('.', ','));
  const lossPercent = $derived(
    stats && stats.packetsReceived + stats.packetsLost > 0
      ? (stats.packetsLost / (stats.packetsReceived + stats.packetsLost)) * 100
      : null,
  );
</script>

<div class="page">
  <header>
    <div class="brand">
      <div class="logo"><Icon name="broadcast" size={16} /></div>
      <span>Screen4Stream</span>
    </div>
    <span class="divider"></span>
    <div class="title">{title || 'Демонстрация экрана'}</div>
    <span class="badge" class:live={badge.live}>{badge.text}</span>
  </header>

  <main>
    <div class="player" bind:this={player}>
      <!-- Видео начинает играть без звука: браузеры не дают включать звук без действия пользователя. -->
      <video bind:this={video} autoplay playsinline muted class:hidden={phase !== 'live'}></video>

      {#if phase === 'live'}
        {#if showStats && stats}
          <dl class="stats">
            <dt>Разрешение</dt><dd>{stats.width ?? '—'} × {stats.height ?? '—'}</dd>
            <dt>Кадры</dt><dd>{format(stats.fps)} fps</dd>
            <dt>Битрейт</dt><dd>{format(stats.bitrateMbps, 1)} Мбит/с</dd>
            <dt>Кодек</dt><dd>{stats.codec ?? '—'}</dd>
            <dt>RTT</dt><dd>{format(stats.rttMs)} мс</dd>
            <dt>Потери</dt><dd>{format(lossPercent, 1)} %</dd>
            <dt>Буфер</dt><dd>{format(stats.jitterBufferMs)} мс</dd>
          </dl>
        {/if}

        <div class="controls">
          <button class="control" type="button" onclick={toggleMute} aria-label={muted ? 'Включить звук' : 'Выключить звук'} title={muted ? 'Включить звук' : 'Выключить звук'}>
            <Icon name={muted ? 'muted' : 'volume'} size={20} />
          </button>
          <span class="live-label"><span class="dot"></span>Прямой эфир</span>
          <span class="spacer"></span>
          <button class="control" class:active={showStats} type="button" onclick={() => (showStats = !showStats)} aria-pressed={showStats} aria-label="Статистика соединения" title="Статистика соединения">
            <Icon name="activity" size={20} />
          </button>
          <button class="control" type="button" onclick={toggleFullscreen} aria-label="Во весь экран" title="Во весь экран">
            <Icon name="fullscreen" size={20} />
          </button>
        </div>
      {:else}
        <div class="message">
          {#if phase === 'connecting'}
            <div class="spinner"></div>
          {:else}
            <div class="message-icon"><Icon name={phase === 'failed' ? 'alert' : 'monitor'} size={32} /></div>
          {/if}
          <div class="message-title">{messages[phase][0]}</div>
          <div class="message-text">{messages[phase][1]}</div>
          {#if phase === 'failed'}
            <button class="btn btn-primary btn-large" type="button" onclick={() => viewer?.retry()}>Попробовать снова</button>
          {/if}
        </div>
      {/if}
    </div>

    <p class="footnote">Смотрите прямо в браузере — устанавливать ничего не нужно. Качество подстраивается под вашу сеть.</p>
  </main>
</div>

<style>
  .page {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-darkest);
  }

  header {
    min-height: 56px;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 16px;
    padding: 8px 16px;
    background: var(--bg-sidebar);
    border-bottom: 1px solid var(--border-dark);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    font-weight: 700;
    font-size: 15px;
  }

  .logo {
    width: 30px;
    height: 30px;
    border-radius: 9px;
    background: var(--accent);
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .divider {
    width: 1px;
    height: 24px;
    background: var(--border);
  }

  .title {
    flex: 1 1 200px;
    min-width: 0;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge {
    padding: 4px 10px;
    border-radius: 4px;
    background: var(--bg-active);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
  }

  .badge.live {
    background: var(--live);
    color: #fff;
  }

  main {
    flex: 1;
    width: 100%;
    max-width: 1200px;
    margin: 0 auto;
    padding: 24px 16px;
  }

  .player {
    position: relative;
    aspect-ratio: 16 / 9;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: #000;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .player:fullscreen {
    border-radius: 0;
  }

  video {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  video.hidden {
    display: none;
  }

  .stats {
    position: absolute;
    top: 12px;
    left: 12px;
    min-width: 220px;
    margin: 0;
    padding: 12px 14px;
    border-radius: var(--radius);
    background: rgba(0, 0, 0, 0.78);
    display: grid;
    grid-template-columns: auto auto;
    gap: 4px 16px;
    font-family: 'Cascadia Mono', Consolas, monospace;
    font-size: 12px;
  }

  .stats dt {
    color: var(--text-secondary);
  }

  .stats dd {
    margin: 0;
    text-align: right;
  }

  .controls {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.72);
  }

  .control {
    width: 40px;
    height: 40px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .control:hover,
  .control.active {
    background: var(--bg-active);
  }

  .live-label {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 8px;
    font-size: 13px;
    font-weight: 600;
  }

  .live-label .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #e5484d;
  }

  .spacer {
    flex: 1;
  }

  .message {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    padding: 24px;
    text-align: center;
  }

  .message-icon {
    width: 72px;
    height: 72px;
    border-radius: 50%;
    background: var(--bg-card);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .message-title {
    font-size: 20px;
    font-weight: 700;
  }

  .message-text {
    max-width: 420px;
    color: var(--text-secondary);
  }

  .spinner {
    width: 44px;
    height: 44px;
    border: 4px solid var(--border);
    border-top-color: var(--accent-text);
    border-radius: 50%;
    animation: spin 0.9s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .footnote {
    margin: 14px 2px 0;
    font-size: 12px;
    color: var(--text-muted);
  }

  /* Телефон: компактная статистика, без разделителя в шапке. */
  @media (max-width: 600px) {
    .divider {
      display: none;
    }

    main {
      padding: 12px 0;
    }

    .player {
      border-radius: 0;
    }

    .stats {
      top: 8px;
      left: 8px;
      min-width: 0;
      padding: 8px 10px;
      gap: 2px 12px;
      font-size: 10px;
    }

    .footnote {
      padding: 0 16px;
    }
  }
</style>
