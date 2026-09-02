import type {
  LaunchRequestedEvent,
  PrivilegeStatus,
  RuntimeNotice,
  RuntimeSnapshot,
} from "./types/generated/bindings";

export type StatusTone = "positive" | "warning" | "critical" | "neutral";

export interface StatusPresentation {
  label: string;
  detail: string;
  tone: StatusTone;
}

export interface VisibleNotice {
  key: string;
  summary: string;
  detail: string | null;
  tone: "warning" | "critical";
}

export function runtimeErrorMessage(error: unknown): string {
  return error instanceof Error && error.message
    ? error.message
    : "无法读取本地运行时状态。";
}

export function mergeLatestLaunch(
  snapshot: RuntimeSnapshot,
  launch: LaunchRequestedEvent | null,
): RuntimeSnapshot {
  return launch ? { ...snapshot, latestSecondaryLaunch: launch } : snapshot;
}

export function mergeRuntimeNotices(
  snapshot: RuntimeSnapshot,
  incoming: RuntimeNotice[],
): RuntimeSnapshot {
  if (incoming.length === 0) {
    return snapshot;
  }
  const notices = [...snapshot.notices];
  for (const notice of incoming) {
    if (
      !notices.some(
        (current) => current.code === notice.code && current.detail === notice.detail,
      )
    ) {
      notices.push(notice);
    }
  }
  return { ...snapshot, notices: notices.slice(-8) };
}

export function describePrivilege(status: PrivilegeStatus): StatusPresentation {
  switch (status.state) {
    case "elevated":
      return {
        label: "已获得管理员权限",
        detail: "当前进程令牌已通过系统检查。",
        tone: "positive",
      };
    case "notElevated":
      return {
        label: "未以管理员身份运行",
        detail: "请关闭应用并以管理员身份重新启动；获得权限前不会执行系统写入。",
        tone: "critical",
      };
    case "unavailable": {
      const evidence = status.failure
        ? `${status.failure.stage}，系统代码 ${status.failure.code}`
        : "未返回系统诊断信息";
      return {
        label: "权限状态未知",
        detail: evidence,
        tone: "warning",
      };
    }
  }
}

export function describeRecovery(count: number): StatusPresentation {
  if (count === 0) {
    return {
      label: "未发现异常会话",
      detail: "当前启动前没有遗留的会话标记。",
      tone: "positive",
    };
  }
  return {
    label: `发现 ${count} 个异常会话标记`,
    detail: "这只表示应用上次未正常退出，暂未执行系统恢复。",
    tone: "warning",
  };
}

export function describeLaunch(launch: LaunchRequestedEvent): StatusPresentation {
  if (launch.notices.length > 0) {
    return {
      label: "已回退为普通启动",
      detail: "启动参数存在问题，请查看下方警告。",
      tone: "warning",
    };
  }
  if (launch.request.startHidden) {
    return {
      label: "后台启动",
      detail: "窗口按 /tray 请求创建为隐藏状态。",
      tone: "neutral",
    };
  }
  if (launch.request.navigationPath) {
    return {
      label: "带导航目标启动",
      detail: "目标已通过传输校验，将由后续路由层处理。",
      tone: "neutral",
    };
  }
  return {
    label: "普通启动",
    detail: "窗口按默认方式显示。",
    tone: "neutral",
  };
}

export function collectVisibleNotices(snapshot: RuntimeSnapshot): VisibleNotice[] {
  const runtime = snapshot.notices.map((notice, index) => ({
    key: `runtime-${notice.code}-${index}`,
    summary: notice.summary,
    detail: notice.detail,
    tone: notice.code === "elevationProbeFailed" ? "critical" : "warning",
  })) satisfies VisibleNotice[];

  const launchGroups: Array<[string, LaunchRequestedEvent | null]> = [
    ["primary", snapshot.primaryLaunch],
    ["secondary", snapshot.latestSecondaryLaunch],
  ];
  const launches = launchGroups.flatMap(([source, launch]) =>
    (launch?.notices ?? []).map((notice, index) => ({
      key: `${source}-${notice.code}-${notice.argumentPosition ?? "none"}-${index}`,
      summary: notice.summary,
      detail: notice.detail,
      tone: "warning" as const,
    })),
  );

  return [...runtime, ...launches];
}
