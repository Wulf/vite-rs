import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  build: {
    rollupOptions: {
      input: ['app/index.html', 'app/pack1.ts'],
    },
    manifest: true, // **IMPORTANT**: this is required.
    outDir: './dist', // this is the default value
  },
  publicDir: './public', // this is the default value
})
