import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import legacy from '@vitejs/plugin-legacy';
import fs from 'fs';
import path from 'path';

let appVersion = '1.4.7';
try {
  const versionPath = path.resolve(__dirname, '../VERSION');
  if (fs.existsSync(versionPath)) {
    appVersion = fs.readFileSync(versionPath, 'utf-8').trim();
  }
} catch (e) {}

export default defineConfig({
  define: {
    'import.meta.env.VITE_APP_VERSION': JSON.stringify(`v${appVersion}`)
  },
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
