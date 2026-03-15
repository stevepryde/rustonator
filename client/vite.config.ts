import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

function appendSources(base: string[], extra: string): string[] {
  const values = extra
    .split(/\s+/)
    .map((value) => value.trim())
    .filter(Boolean);

  return [...base, ...values];
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");

  const connectSrc = appendSources(["'self'"], env.VITE_CSP_CONNECT_SRC || "*").join(" ");
  const scriptSrc = appendSources(
    ["'self'", "https://ajax.googleapis.com", "https://static.cloudflareinsights.com", "'unsafe-inline'", "'unsafe-eval'"],
    env.VITE_CSP_SCRIPT_SRC || ""
  ).join(" ");
  const styleSrc = appendSources(["'self'", "https://fonts.googleapis.com", "'unsafe-inline'"], env.VITE_CSP_STYLE_SRC || "").join(" ");
  const imgSrc = appendSources(["'self'", "data:", "blob:"], env.VITE_CSP_IMG_SRC || "").join(" ");
  const fontSrc = appendSources(["'self'", "https://fonts.gstatic.com"], env.VITE_CSP_FONT_SRC || "").join(" ");
  const mediaSrc = appendSources(["*"], env.VITE_CSP_MEDIA_SRC || "").join(" ");

  const csp = [
    "default-src 'self'",
    "base-uri 'self'",
    "form-action 'self'",
    `img-src ${imgSrc}`,
    `style-src ${styleSrc}`,
    `media-src ${mediaSrc}`,
    `font-src ${fontSrc}`,
    `script-src ${scriptSrc}`,
    `connect-src ${connectSrc}`,
  ].join("; ");

  return {
    root: "src",
    publicDir: "../public",
    base: env.VITE_APP_BASE || "./",
    plugins: [
      react(),
      tailwindcss(),
      {
        name: "inject-csp-meta",
        transformIndexHtml(html) {
          return html.replace("__APP_CSP__", csp);
        },
      },
    ],
    build: {
      outDir: "../dist",
      emptyOutDir: true,
    },
  };
});
