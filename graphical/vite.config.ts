import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import electron from 'vite-plugin-electron'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    electron({
      entry: 'electron/main.ts',
    }),
  ],
  server: {
    port: 5173,
    strictPort: false,
    host: true,
    fs: {
      strict: false,
      allow: ['..']
    }
  }
})
