import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  build: {
    rollupOptions: {
      input: ["index.html"],
    },
  },
  server: {
    hmr: {
      port: 21012,
    },
  },
});
