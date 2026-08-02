import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
      vue(),
  ],
  base: '/sologger/',
  build: {
    outDir: 'dist',
    rollupOptions: {
      input: {
        main: './index.html',
        error: './error.html'
      }
    }
  },
  assetsInclude: ['**/*.svg'],
  test: {
    environment: 'node',
    include: ['src/tests/**/*.test.js']
  }
})

