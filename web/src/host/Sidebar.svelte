<script>
  import Icon from '../lib/Icon.svelte';
  import { openInBrowser } from '../lib/api.js';

  let { live, connectionLost, shareUrl } = $props();

  // Окно Tauri открывает интерфейс с ?app=1 — только там нужна кнопка «Открыть в браузере».
  const insideApp = new URLSearchParams(location.search).has('app');
  const serverAddress = $derived(shareUrl ? new URL(shareUrl).host : '');
</script>

<aside>
  <div class="brand">
    <div class="logo"><Icon name="broadcast" /></div>
    <span>Screen4Stream</span>
  </div>

  <nav aria-label="Разделы">
    <a class="nav-item active" href="/" aria-current="page">
      <Icon name="monitor" />
      Трансляция
      {#if live}<span class="live-dot" title="В эфире"></span>{/if}
    </a>
  </nav>

  <div class="spacer"></div>

  <div class="server">
    <div class="server-icon">
      <Icon name="monitor" size={16} />
      <span class="status-dot" class:offline={connectionLost}></span>
    </div>
    <div class="server-text">
      <div class="server-title">{connectionLost ? 'Нет связи с сервером' : 'Сервер запущен'}</div>
      <div class="server-address">{serverAddress}</div>
    </div>
    {#if insideApp}
      <button
        class="icon-btn"
        type="button"
        onclick={openInBrowser}
        aria-label="Открыть в браузере"
        title="Открыть в браузере"
      >
        <Icon name="external" />
      </button>
    {/if}
  </div>
</aside>

<style>
  aside {
    flex: 1 1 232px;
    display: flex;
    flex-direction: column;
    background: var(--bg-sidebar);
  }

  .brand {
    height: 56px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 16px;
    border-bottom: 1px solid var(--border-dark);
    font-size: 16px;
    font-weight: 700;
  }

  .logo {
    width: 32px;
    height: 32px;
    border-radius: 10px;
    background: var(--accent);
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px 8px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 40px;
    padding: 0 10px;
    border-radius: 6px;
    color: var(--text-secondary);
    font-weight: 500;
    text-decoration: none;
  }

  .nav-item.active {
    background: var(--bg-active);
    color: var(--text);
  }

  .live-dot {
    margin-left: auto;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--live);
  }

  .spacer {
    flex: 1;
  }

  .server {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    background: var(--bg-footer);
  }

  .server-icon {
    position: relative;
    width: 32px;
    height: 32px;
    flex: none;
    border-radius: 50%;
    background: var(--bg-hover);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .status-dot {
    position: absolute;
    right: -2px;
    bottom: -2px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--online);
    border: 3px solid var(--bg-footer);
  }

  .status-dot.offline {
    background: var(--live);
  }

  .server-text {
    flex: 1;
    min-width: 0;
  }

  .server-title {
    font-size: 13px;
    font-weight: 600;
  }

  .server-address {
    font-size: 12px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
