import { resolve } from 'node:path';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const root = import.meta.dirname;

export default defineConfig({
  root,
  plugins: [svelte()],
  server: {
    port: 5173,
    // API и signaling отдаёт Rust-сервер, его нужно запустить рядом:
    //   cargo run -p s4s-server
    proxy: { '/api': 'http://127.0.0.1:8080' },
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    // Две страницы: интерфейс ведущего и страница зрителя.
    rolldownOptions: {
      input: {
        main: resolve(root, 'index.html'),
        watch: resolve(root, 'watch.html'),
      },
    },
  },
});
