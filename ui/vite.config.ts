import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Tauri serves the popover from this dev server, then from ui/dist in builds.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { outDir: "dist", emptyOutDir: true, target: "safari16" },
});
