import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      // Tauri expects static files in the dist dir
      pages: 'dist',
      assets: 'dist',
      fallback: 'index.html', // SPA mode
      precompress: false,
      strict: false
    }),
    // Disable SSR for Tauri — it's a static SPA
    prerender: {
      handleHttpError: 'warn'
    }
  }
};

export default config;
