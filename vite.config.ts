import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Puerto fijo: Tauri espera exactamente esta URL en desarrollo.
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 1420, strictPort: true, watch: { ignored: ["**/src-tauri/**"] } },
});
