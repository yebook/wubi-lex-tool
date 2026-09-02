import "i18next";

import type { zhCN } from "./resources/zh-CN";

declare module "i18next" {
  interface CustomTypeOptions {
    defaultNS: "common";
    resources: typeof zhCN;
    returnNull: false;
  }
}
