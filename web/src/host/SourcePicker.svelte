<!--
  Окно выбора источника: вкладки «Приложения» / «Экраны», миниатюры и настройки.
-->
<script>
  import { onMount, untrack } from 'svelte';
  import Icon from '../lib/Icon.svelte';
  import { getSources, thumbnailUrl } from '../lib/api.js';

  let { initialSettings, live, currentSourceId, busy, error, onCancel, onConfirm } = $props();

  // Окно создаётся заново при каждом открытии, поэтому начальные значения берём один раз.
  const initial = untrack(() => ({ ...initialSettings, sourceId: currentSourceId }));

  let tab = $state(initial.sourceId?.startsWith('mon-') ? 'monitors' : 'windows');
  let sources = $state({ windows: [], monitors: [] });
  let loading = $state(true);
  let loadError = $state('');
  let selectedId = $state(initial.sourceId ?? null);

  let height = $state(initial.height);
  let fps = $state(initial.fps);
  let cursor = $state(initial.cursor);
  let audio = $state(initial.audio);

  const resolutions = [
    { label: '720p', value: 720 },
    { label: '1080p', value: 1080 },
    { label: 'Исходное', value: null },
  ];
  const framerates = [15, 30, 60];

  const list = $derived(tab === 'windows' ? sources.windows : sources.monitors);
  const selected = $derived(
    [...sources.windows, ...sources.monitors].find((s) => s.id === selectedId) ?? null,
  );

  async function load() {
    loading = true;
    loadError = '';
    try {
      const result = await getSources();
      // Адрес миниатюры фиксируем при загрузке, иначе она перезапрашивалась бы при каждой отрисовке.
      for (const source of [...result.windows, ...result.monitors]) {
        source.thumbnail = thumbnailUrl(source.id, 320);
      }
      sources = result;
    } catch (err) {
      loadError = err.message;
    } finally {
      loading = false;
    }
  }

  onMount(load);

  function confirm() {
    if (selected) {
      onConfirm(selected.id, { height, fps, cursor, audio });
    }
  }

  function onKeydown(event) {
    if (event.key === 'Escape') onCancel();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="overlay">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="picker-title">
    <div class="dialog-header">
      <h2 id="picker-title">Демонстрация экрана</h2>
      <button class="icon-btn" type="button" onclick={load} aria-label="Обновить список" title="Обновить список">
        <Icon name="refresh" />
      </button>
      <button class="icon-btn" type="button" onclick={onCancel} aria-label="Закрыть">
        <Icon name="close" size={20} />
      </button>
    </div>

    <div class="tabs">
      <button type="button" class:active={tab === 'windows'} aria-pressed={tab === 'windows'} onclick={() => (tab = 'windows')}>
        Приложения
      </button>
      <button type="button" class:active={tab === 'monitors'} aria-pressed={tab === 'monitors'} onclick={() => (tab = 'monitors')}>
        Экраны
      </button>
    </div>

    <div class="grid">
      {#if loading}
        <p class="empty">Загрузка…</p>
      {:else if loadError}
        <p class="empty">{loadError}</p>
      {:else if list.length === 0}
        <p class="empty">Нет доступных окон</p>
      {:else}
        {#each list as source (source.id)}
          <button
            type="button"
            class="source"
            class:selected={source.id === selectedId}
            aria-pressed={source.id === selectedId}
            onclick={() => (selectedId = source.id)}
          >
            <div class="thumb">
              {#if source.minimized}
                <span class="thumb-note">Окно свёрнуто</span>
              {:else}
                <img src={source.thumbnail} alt="" loading="lazy" />
              {/if}
            </div>
            <div class="source-title">{source.title}</div>
            <div class="source-subtitle">{source.subtitle}</div>
          </button>
        {/each}
      {/if}
    </div>

    {#if selected?.minimized}
      <div class="banner banner-warning message">
        Окно свёрнуто — разверните его, иначе зрители увидят чёрный экран.
      </div>
    {/if}
    {#if error}
      <div class="banner banner-error message">{error}</div>
    {/if}

    <div class="settings">
      <fieldset>
        <legend class="section-label">Разрешение</legend>
        <div class="pills">
          {#each resolutions as option}
            <button type="button" class="pill" class:active={height === option.value} aria-pressed={height === option.value} onclick={() => (height = option.value)}>
              {option.label}
            </button>
          {/each}
        </div>
      </fieldset>

      <fieldset>
        <legend class="section-label">Частота кадров</legend>
        <div class="pills">
          {#each framerates as option}
            <button type="button" class="pill" class:active={fps === option} aria-pressed={fps === option} onclick={() => (fps = option)}>
              {option}
            </button>
          {/each}
        </div>
      </fieldset>

      <div class="checks">
        <label><input type="checkbox" bind:checked={cursor} /> Показывать курсор</label>
        <label><input type="checkbox" bind:checked={audio} /> Транслировать звук</label>
      </div>
    </div>

    <div class="footer">
      <button class="btn btn-ghost" type="button" onclick={onCancel}>Отмена</button>
      <button class="btn btn-primary" type="button" onclick={confirm} disabled={!selected || busy}>
        {busy ? 'Запуск…' : live ? 'Переключить' : 'В эфир'}
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    background: rgba(0, 0, 0, 0.72);
  }

  .dialog {
    width: 100%;
    max-width: 780px;
    max-height: calc(100vh - 32px);
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--bg-main);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 18px 16px 4px 20px;
  }

  h2 {
    flex: 1;
    margin: 0;
    font-size: 20px;
  }

  .tabs {
    display: flex;
    gap: 20px;
    padding: 0 20px;
    border-bottom: 1px solid var(--border);
  }

  .tabs button {
    min-height: 44px;
    padding: 0 2px;
    border: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--text-secondary);
    font-weight: 600;
  }

  .tabs button.active {
    border-bottom-color: var(--accent);
    color: var(--text);
  }

  .grid {
    flex: 1;
    min-height: 160px;
    overflow: auto;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    align-content: start;
    gap: 12px;
    padding: 16px 20px;
  }

  .empty {
    grid-column: 1 / -1;
    margin: 32px 0;
    text-align: center;
    color: var(--text-muted);
  }

  .source {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px;
    border: 2px solid transparent;
    border-radius: var(--radius);
    background: var(--bg-card);
    text-align: left;
  }

  .source:hover {
    background: var(--bg-hover);
  }

  .source.selected {
    border-color: var(--accent);
    background: var(--bg-selected);
  }

  .thumb {
    aspect-ratio: 16 / 10;
    margin-bottom: 4px;
    border-radius: 4px;
    overflow: hidden;
    background: #111214;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .thumb-note {
    font-size: 12px;
    color: var(--text-muted);
  }

  .source-title,
  .source-subtitle {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .source-title {
    font-size: 13px;
    font-weight: 600;
  }

  .source-subtitle {
    font-size: 12px;
    color: var(--text-muted);
  }

  .message {
    margin: 0 20px 12px;
  }

  .settings {
    display: flex;
    flex-wrap: wrap;
    gap: 16px 32px;
    padding: 16px 20px;
    background: var(--bg-card);
  }

  fieldset {
    margin: 0;
    padding: 0;
    border: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  legend {
    padding: 0;
    margin-bottom: 8px;
  }

  .pills {
    display: flex;
    gap: 8px;
  }

  .pill {
    min-height: 36px;
    padding: 0 14px;
    border: 1px solid #4e5058;
    border-radius: 6px;
    background: var(--bg-main);
    color: #dbdee1;
    font-size: 13px;
    font-weight: 500;
  }

  .pill.active {
    border-color: var(--accent);
    background: var(--accent);
    color: #fff;
    font-weight: 600;
  }

  .checks {
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
  }

  .checks label {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 32px;
    font-size: 13px;
    color: #dbdee1;
    cursor: pointer;
  }

  .checks input {
    width: 18px;
    height: 18px;
    margin: 0;
    accent-color: var(--accent);
  }

  .footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 14px 20px;
    background: var(--bg-card);
    border-top: 1px solid var(--border-dark);
  }
</style>
