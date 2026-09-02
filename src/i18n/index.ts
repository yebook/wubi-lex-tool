import { createInstance } from "i18next";
import { initReactI18next } from "react-i18next";

import type { AppLocale } from "../types/generated/bindings";
import { zhCN } from "./resources/zh-CN";

export const bundledResources = {
  "zh-CN": zhCN,
} satisfies Record<AppLocale, typeof zhCN>;

export const i18n = createInstance();

void i18n.use(initReactI18next).init({
  resources: bundledResources,
  lng: "zh-CN",
  fallbackLng: "zh-CN",
  supportedLngs: Object.keys(bundledResources),
  defaultNS: "common",
  ns: Object.keys(zhCN),
  interpolation: { escapeValue: false },
  initAsync: false,
  returnNull: false,
});
