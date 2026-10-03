import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// Tauri dev server host — must match tauri.conf.json devUrl
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [sveltekit()],
  // Prevent Vite from obscuring Rust errors
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421
        }
      : undefined,
    watch: {
      // Tell Vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**']
    }
  },
  // Env variables starting with VITE_ or TAURI_ are exposed
  envPrefix: ['VITE_', 'TAURI_ENV_*']
});
