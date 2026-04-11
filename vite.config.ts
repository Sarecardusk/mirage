import path from "node:path";
import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig, loadEnv } from "vite";
import vue from "@vitejs/plugin-vue";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  return {
    plugins: [vue(), tailwindcss()],

    resolve: {
      alias: {
        "@": path.resolve(__dirname, "src"),
      },
    },

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent Vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
      port: 1420,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: "ws",
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        // 3. tell Vite to ignore watching `src-tauri`
        ignored: ["**/src-tauri/**"],
      },
    },
    test: {
      globals: true,
      include: ["src/**/*.test.ts", "src/**/*.spec.ts"],
      passWithNoTests: true,
      env: {
        MIRAGE_COMPLETION_ENDPOINT:
          env.MIRAGE_COMPLETION_ENDPOINT ?? process.env.MIRAGE_COMPLETION_ENDPOINT ?? "",
        MIRAGE_COMPLETION_API_KEY:
          env.MIRAGE_COMPLETION_API_KEY ?? process.env.MIRAGE_COMPLETION_API_KEY ?? "",
        MIRAGE_COMPLETION_MODEL:
          env.MIRAGE_COMPLETION_MODEL ?? process.env.MIRAGE_COMPLETION_MODEL ?? "",
      },
    },
  };
});
