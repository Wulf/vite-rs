import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  build: {
    rollupOptions: {
      input: [path.resolve(__dirname, 'app/index.html'), path.resolve(__dirname, 'app/pack1.ts')],
    },
    manifest: true, // **IMPORTANT**: this is required.
    outDir: path.resolve(__dirname, './dist'),
  },
  publicDir: path.resolve(__dirname, './public'),
})
