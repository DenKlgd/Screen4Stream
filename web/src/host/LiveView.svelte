<!--
  Экран «В эфире»: превью, ссылка для зрителей, параметры трансляции.

  Превью — это миниатюра источника, обновляемая раз в 2 секунды. Смотреть
  собственный WebRTC-поток было бы плавнее, но webrtcsink кодирует видео
  отдельно для каждого зрителя, и превью удвоило бы нагрузку на видеокарту.
-->
<script>
  import Icon from '../lib/Icon.svelte';
  import { thumbnailUrl } from '../lib/api.js';

  let { status, onChangeSource, onStop } = $props();

  const sourceId = $derived(status.source.id);
  const settings = $derived(status.settings);

  let preview = $state('');
  let copied = $state(false);
  let linkInput;

  // Новую картинку подменяем только после загрузки — без мигания.
  $effect(() => {
    const id = sourceId;
    let active = true;
    const load = () => {
      const url = thumbnailUrl(id, 960);
      const image = new Image();
      image.onload = () => {
        if (active) preview = url;
      };
      image.src = url;
    };
    load();
    const timer = setInterval(load, 2000);
    return () => {
      active = false;
      clearInterval(timer);
    };
  });

  async function copyLink() {
    try {
      await navigator.clipboard.writeText(status.shareUrl);
    } catch {
      // Старый способ — если Clipboard API недоступен.
      linkInput.select();
      document.execCommand('copy');
    }
    copied = true;
    setTimeout(() => (copied = false), 1600);
  }

  const resolutionLabel = $derived(settings.height ? `${settings.height}p` : 'Исходное');
</script>

<div class="live">
  {#if status.noVideo}
    <div class="banner banner-warning">
      Источник пока не отдаёт изображение, поэтому зрители его не видят.
      Если окно перекрыто другими — выведите его на передний план:
      некоторые программы (например, Windows Terminal) не рисуют себя, пока их не видно.
    </div>
  {/if}

  <section class="preview-card">
    <div class="stage">
      {#if preview}
        <img src={preview} alt="Превью: {status.source.title}" />
      {/if}
      <span class="badge badge-live">В ЭФИРЕ</span>
      <span class="badge badge-viewers" title="Зрители">
        <Icon name="eye" size={14} />
        {status.viewers}
      </span>
      <span class="badge badge-format">{resolutionLabel} · {settings.fps} fps</span>
    </div>

    <div class="source-row">
      <div class="source-text">
        <div class="source-title">{status.source.title}</div>
        <div class="source-subtitle">{status.source.subtitle}</div>
      </div>
      <button class="btn btn-secondary" type="button" onclick={onChangeSource}>
        <Icon name="swap" size={16} />
        Сменить источник
      </button>
      <button class="btn btn-danger" type="button" onclick={onStop}>
        <Icon name="stop" size={16} />
        Остановить
      </button>
    </div>
  </section>

  <section class="card">
    <label class="section-label" for="share-link">Ссылка для зрителей</label>
    <div class="link-row">
      <input id="share-link" bind:this={linkInput} value={status.shareUrl} readonly />
      <button class="btn" class:copied type="button" onclick={copyLink}>
        <Icon name={copied ? 'check' : 'copy'} size={16} />
        {copied ? 'Скопировано' : 'Копировать'}
      </button>
    </div>
    <p class="hint">Откройте на любом устройстве в этой сети — ничего устанавливать не нужно.</p>
  </section>

  <section class="tiles">
    <div class="tile">
      <div class="tile-label">Зрители</div>
      <div class="tile-value">{status.viewers}</div>
    </div>
    <div class="tile">
      <div class="tile-label">Разрешение</div>
      <div class="tile-value">{resolutionLabel}</div>
    </div>
    <div class="tile">
      <div class="tile-label">Частота кадров</div>
      <div class="tile-value">{settings.fps} fps</div>
    </div>
    <div class="tile">
      <div class="tile-label">Звук</div>
      <div class="tile-value">{settings.audio ? 'Системный' : 'Выключен'}</div>
    </div>
  </section>
</div>

<style>
  .live {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 1100px;
    width: 100%;
    padding: 24px;
  }

  .preview-card,
  .card {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .stage {
    position: relative;
    aspect-ratio: 16 / 9;
    background: #111214;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .stage img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .badge {
    position: absolute;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.7);
    font-size: 12px;
    font-weight: 600;
  }

  .badge-live {
    top: 12px;
    left: 12px;
    background: var(--live);
    font-weight: 700;
    letter-spacing: 0.04em;
  }

  .badge-viewers {
    top: 12px;
    right: 12px;
  }

  .badge-format {
    bottom: 12px;
    right: 12px;
  }

  .source-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
  }

  .source-text {
    flex: 1 1 200px;
    min-width: 0;
  }

  .source-title {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .source-subtitle {
    font-size: 12px;
    color: var(--text-muted);
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
  }

  .link-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .link-row input {
    flex: 1 1 260px;
    min-width: 0;
    min-height: 40px;
    padding: 0 12px;
    border: 0;
    border-radius: 6px;
    background: var(--bg-darkest);
    color: #dbdee1;
    font-family: 'Cascadia Mono', Consolas, monospace;
    font-size: 13px;
  }

  .link-row .btn {
    background: var(--accent);
    color: #fff;
  }

  .link-row .btn.copied {
    background: var(--success);
  }

  .hint {
    margin: 0;
    font-size: 12px;
    color: var(--text-muted);
  }

  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
  }

  .tile {
    padding: 12px 14px;
    border-radius: 10px;
    background: var(--bg-card);
  }

  .tile-label {
    font-size: 12px;
    color: var(--text-muted);
  }

  .tile-value {
    font-size: 16px;
    font-weight: 600;
  }
</style>
