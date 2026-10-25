<script>
  import Icon from '../lib/Icon.svelte';

  let { onStart, disabled } = $props();
</script>

<div class="idle">
  <div class="content">
    <div class="circle"><Icon name="monitor" size={40} /></div>
    <h2>Поделитесь экраном</h2>
    <p>Выберите окно или весь экран — зрители откроют ссылку в браузере и сразу увидят трансляцию.</p>
    <button class="btn btn-primary btn-large" type="button" onclick={onStart} {disabled}>
      <Icon name="broadcast" />
      Начать демонстрацию
    </button>
    <p class="hint">Ссылка для зрителей появится после начала трансляции.</p>
  </div>
</div>

<style>
  .idle {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
  }

  .content {
    max-width: 460px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    text-align: center;
  }

  .circle {
    width: 88px;
    height: 88px;
    border-radius: 50%;
    background: var(--bg-card);
    color: var(--accent-text);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  h2 {
    margin: 0;
    font-size: 24px;
  }

  p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 15px;
  }

  .btn {
    margin-top: 8px;
  }

  .hint {
    color: var(--text-muted);
    font-size: 12px;
  }
</style>
