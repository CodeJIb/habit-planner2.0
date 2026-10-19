import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

import { cloudflare } from "@cloudflare/vite-plugin";

// Tauri expects a fixed dev-server port (see devUrl in src-tauri/tauri.conf.json).
export default defineConfig({
  plugins: [react(), tailwindcss(), cloudflare()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**'] },
  },
})