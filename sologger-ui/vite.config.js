import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
// import vitePluginWasmPack from "vite-plugin-wasm-pack";

// https://vite.dev/config/
export default defineConfig({
  plugins: [
      vue(),
      // vitePluginWasmPack('./public/sologger-log-transformer-wasm')
  ],
  optimizeDeps: {
    include: ['@handsontable/vue3']
  },
  // Add this for error page handling
  // publicDir: 'public',
  // build: {
  //   outDir: 'dist',
  //   assetsDir: 'assets',
  //   rollupOptions: {
  //     input: {
  //       main: './index.html',
  //       error: './error.html'
  //     }
  //   }
  // }
})

