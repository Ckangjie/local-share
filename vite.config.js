import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

// https://vitejs.dev/config/
export default defineConfig({
  base: './',
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  // 防止 Vite 清空 Tauri 调试日志
  clearScreen: false,
  build: {
    emptyOutDir: true
  },
  server: {
    port: 1420,
    strictPort: true,
    host: '127.0.0.1'
  }
})
