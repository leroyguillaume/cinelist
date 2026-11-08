import { minimal2023Preset, type Preset } from "@vite-pwa/assets-generator/config";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";
import { defineConfig } from "vitest/config";

const BACKGROUND = "#192e41";

const icons: Preset = {
  ...minimal2023Preset,
  maskable: { sizes: [512], padding: 0.2, resizeOptions: { background: BACKGROUND } },
  apple: { sizes: [180], padding: 0, resizeOptions: { background: BACKGROUND } },
};

export default defineConfig({
  plugins: [
    react(),
    VitePWA({
      registerType: "prompt",
      injectRegister: false,
      pwaAssets: {
        image: "public/favicon.svg",
        preset: icons,
        overrideManifestIcons: true,
        injectThemeColor: false,
      },
      manifest: {
        id: "/",
        name: "CineList",
        short_name: "CineList",
        lang: "fr",
        start_url: "/",
        display: "standalone",
        background_color: BACKGROUND,
        theme_color: BACKGROUND,
      },
      workbox: {
        globPatterns: ["**/*.{js,css,html,svg,png,ico}"],
        // Navigations outside the single-page app must reach the server rather
        // than the cached index.html: the API, its docs page included, and
        // static-web-server's health check.
        navigateFallbackDenylist: [/^\/api(\/|$)/, /^\/health$/],
        runtimeCaching: [
          {
            urlPattern: ({ url }) => url.origin === "https://fonts.googleapis.com",
            handler: "StaleWhileRevalidate",
            options: { cacheName: "google-fonts-stylesheets" },
          },
          {
            urlPattern: ({ url }) => url.origin === "https://fonts.gstatic.com",
            handler: "CacheFirst",
            options: {
              cacheName: "google-fonts-webfonts",
              cacheableResponse: { statuses: [0, 200] },
              expiration: { maxEntries: 30, maxAgeSeconds: 365 * 24 * 60 * 60 },
            },
          },
        ],
      },
    }),
  ],
  server: {
    proxy: {
      "/api": "http://localhost:8080",
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
  },
});
