import type { TFunction } from "i18next";

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

export function runtimeErrorMessage(
  error: unknown,
  t: TFunction<"runtime">,
): string {
  return error instanceof Error && error.message
    ? error.message
    : t("loadError.fallback");
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
        (current) =>
          current.code === notice.code && current.detail === notice.detail,
      )
    ) {
      notices.push(notice);
    }
  }
  return { ...snapshot, notices: notices.slice(-8) };
}

export function describePrivilege(
  status: PrivilegeStatus,
  t: TFunction<"runtime">,
): StatusPresentation {
  switch (status.state) {
    case "elevated":
      return {
        label: t("privilege.elevated.label"),
        detail: t("privilege.elevated.detail"),
        tone: "positive",
      };
    case "notElevated":
      return {
        label: t("privilege.notElevated.label"),
        detail: t("privilege.notElevated.detail"),
        tone: "critical",
      };
    case "unavailable": {
      const evidence = status.failure
        ? t("privilege.unavailable.evidence", {
            stage: status.failure.stage,
            code: status.failure.code,
          })
        : t("privilege.unavailable.noEvidence");
      return {
        label: t("privilege.unavailable.label"),
        detail: evidence,
        tone: "warning",
      };
    }
  }
}

export function describeRecovery(
  count: number,
  t: TFunction<"runtime">,
): StatusPresentation {
  if (count === 0) {
    return {
      label: t("recovery.clean.label"),
      detail: t("recovery.clean.detail"),
      tone: "positive",
    };
  }
  return {
    label: t("recovery.abnormal.label", { count }),
    detail: t("recovery.abnormal.detail"),
    tone: "warning",
  };
}

export function describeLaunch(
  launch: LaunchRequestedEvent,
  t: TFunction<"runtime">,
): StatusPresentation {
  if (launch.notices.length > 0) {
    return {
      label: t("launch.fallback.label"),
      detail: t("launch.fallback.detail"),
      tone: "warning",
    };
  }
  if (launch.request.startHidden) {
    return {
      label: t("launch.hidden.label"),
      detail: t("launch.hidden.detail"),
      tone: "neutral",
    };
  }
  if (launch.request.navigationPath) {
    return {
      label: t("launch.navigation.label"),
      detail: t("launch.navigation.detail"),
      tone: "neutral",
    };
  }
  return {
    label: t("launch.normal.label"),
    detail: t("launch.normal.detail"),
    tone: "neutral",
  };
}

export function collectVisibleNotices(
  snapshot: RuntimeSnapshot,
): VisibleNotice[] {
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
