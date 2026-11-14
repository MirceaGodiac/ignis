import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The Rust server (`cargo run -p ignis-cli -- serve --dev`) listens on IGNIS_PORT (default 8787).
const api = `http://127.0.0.1:${process.env.IGNIS_PORT ?? "8787"}`;

export default defineConfig({
  plugins: [react()],
  server: { port: Number(process.env.PORT ?? 5173), strictPort: false, proxy: { "/api": { target: api, changeOrigin: false } } },
  build: { target: "es2022", sourcemap: true },
});
