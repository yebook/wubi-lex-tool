import { describe, expect, it } from "vitest";

import { bundledResources, i18n } from ".";

describe("bundled i18n", () => {
  it("initializes synchronously from the zh-CN bundle", () => {
    expect(i18n.isInitialized).toBe(true);
    expect(i18n.language).toBe("zh-CN");
    expect(i18n.t("runtime:recovery.abnormal.label", { count: 2 })).toBe(
      "发现 2 个异常会话标记",
    );
  });

  it("keeps the locale registry complete without network backends", () => {
    expect(Object.keys(bundledResources)).toEqual(["zh-CN"]);
    expect(i18n.services.backendConnector.backend).toBeNull();
  });
});
