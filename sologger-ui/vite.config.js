import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
      vue(),
  ],
  optimizeDeps: {
    include: ['@handsontable/vue3']
  },
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
  assetsInclude: ['**/*.svg']
})

