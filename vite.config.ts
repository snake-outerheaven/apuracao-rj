// SPDX-License-Identifier: GPL-3.0-or-later
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true, host: "localhost" },
  build: { target: "es2021", outDir: "dist" },
  test: { environment: "node", include: ["src/**/*.test.ts"] },
});
