<!--
  Интерфейс ведущего: боковая панель + экран «Не в эфире» или «В эфире»
  + окно выбора источника поверх.
-->
<script>
  import { onMount } from 'svelte';
  import * as api from '../lib/api.js';
  import { loadSettings, saveSettings } from './settings.js';
  import Sidebar from './Sidebar.svelte';
  import IdleView from './IdleView.svelte';
  import LiveView from './LiveView.svelte';
  import SourcePicker from './SourcePicker.svelte';

  // Состояние с сервера (см. Status в crates/server/src/state.rs).
  let status = $state(null);
  let connectionLost = $state(false);
  let pickerOpen = $state(false);
  let starting = $state(false);
  let startError = $state('');
  let settings = $state(loadSettings());

  async function refresh() {
    try {
      status = await api.getStatus();
      connectionLost = false;
    } catch {
      connectionLost = true;
    }
  }

  // Опрашиваем сервер раз в секунду: таймер эфира, число зрителей, ошибки.
  onMount(() => {
    refresh();
    const timer = setInterval(refresh, 1000);
    return () => clearInterval(timer);
  });

  function openPicker() {
    startError = '';
    pickerOpen = true;
  }

  async function goLive(sourceId, newSettings) {
    settings = newSettings;
    saveSettings(newSettings);
    starting = true;
    startError = '';
    try {
      status = await api.startStream({ sourceId, ...newSettings });
      pickerOpen = false;
    } catch (err) {
      startError = err.message;
    } finally {
      starting = false;
    }
  }

  async function stop() {
    status = await api.stopStream();
  }
</script>

<div class="app">
  <Sidebar live={status?.live ?? false} {connectionLost} shareUrl={status?.shareUrl} />

  <main>
    <header>
      <h1>Трансляция</h1>
      {#if status?.live}
        <span class="chip chip-live">
          <span class="dot"></span>
          В ЭФИРЕ · {api.formatDuration(status.elapsedSecs)}
        </span>
      {:else}
        <span class="chip">
          <span class="dot"></span>
          Не в эфире
        </span>
      {/if}
    </header>

    {#if connectionLost}
      <div class="banner banner-error notice">Нет связи с сервером Screen4Stream. Он запущен?</div>
    {:else if status?.lastError}
      <div class="banner banner-warning notice">{status.lastError}</div>
    {/if}

    {#if status?.live}
      <LiveView {status} onChangeSource={openPicker} onStop={stop} />
    {:else}
      <IdleView onStart={openPicker} disabled={!status} />
    {/if}
  </main>

  {#if pickerOpen}
    <SourcePicker
      initialSettings={settings}
      live={status?.live ?? false}
      currentSourceId={status?.source?.id}
      busy={starting}
      error={startError}
      onCancel={() => (pickerOpen = false)}
      onConfirm={goLive}
    />
  {/if}
</div>

<style>
  .app {
    position: relative;
    min-height: 100vh;
    display: flex;
    flex-wrap: wrap;
  }

  main {
    flex: 999 1 560px;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  header {
    min-height: 56px;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    padding: 8px 24px;
    border-bottom: 1px solid #26272b;
  }

  h1 {
    flex: 1;
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .chip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    border-radius: 999px;
    background: var(--bg-card);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  .chip .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #80848e;
  }

  .chip-live {
    background: var(--live);
    color: #fff;
    font-weight: 700;
    letter-spacing: 0.04em;
    font-variant-numeric: tabular-nums;
  }

  .chip-live .dot {
    background: #fff;
  }

  .notice {
    margin: 16px 24px 0;
  }
</style>
