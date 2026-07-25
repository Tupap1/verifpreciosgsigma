import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import legacy from '@vitejs/plugin-legacy';

export default defineConfig({
  plugins: [
    svelte(),
    legacy({
      targets: ['chrome >= 30', 'android >= 4.4', 'firefox >= 30', 'ie >= 11'],
      additionalLegacyPolyfills: ['core-js/stable'],
      renderLegacyChunks: true,
      polyfills: true
    })
  ],
  build: {
    outDir: 'build',
    emptyOutDir: true
  }
});
