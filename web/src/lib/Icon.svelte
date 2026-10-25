<!--
  Линейные иконки 24×24. Использование: <Icon name="monitor" size={18} />
-->
<script>
  let { name, size = 18 } = $props();

  const icons = {
    broadcast: [
      'M14 12a2 2 0 1 1-4 0a2 2 0 1 1 4 0',
      'M16.24 7.76a6 6 0 0 1 0 8.48M7.76 16.24a6 6 0 0 1 0-8.48M19.07 4.93a10 10 0 0 1 0 14.14M4.93 19.07a10 10 0 0 1 0-14.14',
    ],
    monitor: ['M4 3h16a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z', 'M8 21h8M12 17v4'],
    external: ['M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6M15 3h6v6M10 14L21 3'],
    close: ['M18 6L6 18M6 6l12 12'],
    copy: [
      'M11 9h9a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-9a2 2 0 0 1-2-2v-9a2 2 0 0 1 2-2z',
      'M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1',
    ],
    check: ['M20 6L9 17l-5-5'],
    stop: ['M8 6h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2z'],
    swap: ['M23 4v6h-6M1 20v-6h6', 'M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15'],
    refresh: ['M23 4v6h-6', 'M20.49 15a9 9 0 1 1-2.12-9.36L23 10'],
    eye: ['M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z', 'M15 12a3 3 0 1 1-6 0a3 3 0 1 1 6 0'],
    fullscreen: ['M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3'],
    volume: ['M11 5L6 9H2v6h4l5 4V5z', 'M15.54 8.46a5 5 0 0 1 0 7.07M19.07 4.93a10 10 0 0 1 0 14.14'],
    muted: ['M11 5L6 9H2v6h4l5 4V5z', 'M23 9l-6 6M17 9l6 6'],
    activity: ['M22 12h-4l-3 9L9 3l-3 9H2'],
    alert: ['M22 12a10 10 0 1 1-20 0a10 10 0 1 1 20 0', 'M12 8v4M12 16h.01'],
  };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#each icons[name] ?? [] as d}
    <path {d} />
  {/each}
</svg>
