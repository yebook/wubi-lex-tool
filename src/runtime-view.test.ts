import { describe, expect, it } from "vitest";

import { i18n } from "./i18n";
import type { RuntimeSnapshot } from "./types/generated/bindings";
import {
  collectVisibleNotices,
  describeLaunch,
  describePrivilege,
  describeRecovery,
  mergeLatestLaunch,
  mergeRuntimeNotices,
  runtimeErrorMessage,
} from "./runtime-view";

const t = i18n.getFixedT("zh-CN", "runtime");

const baseline: RuntimeSnapshot = {
  privilege: { state: "elevated", failure: null },
  previousAbnormalSessionCount: 0,
  primaryLaunch: {
    request: { startHidden: false, navigationPath: null },
    notices: [],
  },
  latestSecondaryLaunch: null,
  notices: [],
};

describe("runtime status presentation", () => {
  it("keeps actual elevation and recovery evidence visible", () => {
    expect(describePrivilege(baseline.privilege, t).tone).toBe("positive");
    expect(
      describePrivilege(
        {
          state: "unavailable",
          failure: { stage: "OpenProcessToken", code: 5 },
        },
        t,
      ).detail,
    ).toContain("OpenProcessToken");
    expect(describeRecovery(2, t)).toMatchObject({
      tone: "warning",
      label: "发现 2 个异常会话标记",
    });
    expect(
      describePrivilege({ state: "notElevated", failure: null }, t).detail,
    ).toContain("以管理员身份重新启动");
  });

  it("keeps loading failures actionable without assuming an Error object", () => {
    expect(runtimeErrorMessage(new Error("IPC unavailable"), t)).toBe(
      "IPC unavailable",
    );
    expect(runtimeErrorMessage({ reason: "unknown" }, t)).toBe(
      "无法读取本地运行时状态。",
    );
  });

  it("distinguishes hidden, navigation and fallback launches", () => {
    expect(
      describeLaunch(
        {
          request: { startHidden: true, navigationPath: null },
          notices: [],
        },
        t,
      ).label,
    ).toBe("后台启动");
    expect(
      describeLaunch(
        {
          request: { startHidden: false, navigationPath: "/settings/runtime" },
          notices: [],
        },
        t,
      ).label,
    ).toBe("带导航目标启动");
    expect(
      describeLaunch(
        {
          request: { startHidden: false, navigationPath: null },
          notices: [
            {
              code: "unknownArgument",
              summary: "参数无效",
              detail: null,
              argumentPosition: 1,
            },
          ],
        },
        t,
      ).tone,
    ).toBe("warning");
  });

  it("combines runtime and latest launch notices", () => {
    const notices = collectVisibleNotices({
      ...baseline,
      notices: [
        {
          code: "loggingUnavailable",
          summary: "日志不可用",
          detail: "阶段：初始化。",
        },
      ],
      latestSecondaryLaunch: {
        request: { startHidden: false, navigationPath: null },
        notices: [
          {
            code: "missingNavigatePath",
            summary: "缺少目标",
            detail: null,
            argumentPosition: 1,
          },
        ],
      },
    });

    expect(notices.map((notice) => notice.summary)).toEqual([
      "日志不可用",
      "缺少目标",
    ]);
  });

  it("merges a launch received while the initial snapshot is in flight", () => {
    const launch = {
      request: { startHidden: false, navigationPath: "/settings/runtime" },
      notices: [],
    };

    expect(mergeLatestLaunch(baseline, launch).latestSecondaryLaunch).toEqual(
      launch,
    );
    expect(mergeLatestLaunch(baseline, null)).toBe(baseline);
  });

  it("merges live runtime notices without duplicating the authoritative snapshot", () => {
    const notice = {
      code: "trayUnavailable" as const,
      summary: "托盘不可用",
      detail: "失败阶段：create_tray。",
    };
    const snapshot = { ...baseline, notices: [notice] };

    expect(mergeRuntimeNotices(snapshot, [notice])).toEqual(snapshot);
    expect(
      mergeRuntimeNotices(snapshot, [
        {
          code: "windowPersistenceFailed",
          summary: "保存失败",
          detail: null,
        },
      ]).notices,
    ).toHaveLength(2);
  });
});
