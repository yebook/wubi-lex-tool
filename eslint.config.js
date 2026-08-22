import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import tseslint from "typescript-eslint";

export default defineConfig([
  globalIgnores([
    ".agents/**",
    ".claude/**",
    ".codex/**",
    ".trellis/**",
    "dist/**",
    "node_modules/**",
    "target/**",
    "wubi-lex/**",
  ]),
  js.configs.recommended,
  tseslint.configs.recommended,
]);
