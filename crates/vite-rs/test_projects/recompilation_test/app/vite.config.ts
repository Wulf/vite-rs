import { defineConfig } from "vite";
import { globSync } from "glob";

export default defineConfig(() => ({
  build: {
    rollupOptions: {
      input: globSync("*.txt"),
    },
    manifest: true,
  },
}));
