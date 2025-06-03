/// <reference types="vitest" />
import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig(({ mode }) => {
  // Load env file based on `mode` in the current working directory.
  const env = loadEnv(mode, process.cwd(), '')
  
  return {
    plugins: [vue()],
    server: {
      port: 3000,
      host: true
    },
    build: {
      outDir: 'dist'
    },
    define: {
      // Make sure env variables are available in the browser
      __ANTHROPIC_API_KEY__: JSON.stringify(env.VITE_ANTHROPIC_API_KEY || env.ANTHROPIC_API_KEY)
    },
    test: {
      environment: 'jsdom',
      globals: true,
      setupFiles: ['./src/test/setup.js']
    }
  }
})