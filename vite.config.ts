import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

const developmentCspNonce = "d3ViaWxleC12aXRlLWRldg==";

export default defineConfig(({ command }) => {
  const config = {
    plugins: [tailwindcss()],
    server: {
      watch: {
        ignored: ["**/target/**"],
      },
    },
  };

  return command === "serve"
    ? { ...config, html: { cspNonce: developmentCspNonce } }
    : config;
});
