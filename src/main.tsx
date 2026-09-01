import { StrictMode, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";

import { commands, events } from "./types/generated/bindings";
import type {
  LaunchRequestedEvent,
  RuntimeSnapshot,
} from "./types/generated/bindings";
import {
  collectVisibleNotices,
  describeLaunch,
  describePrivilege,
  describeRecovery,
  mergeLatestLaunch,
  runtimeErrorMessage,
} from "./runtime-view";
import type { StatusPresentation } from "./runtime-view";
import { featuresStore } from "./stores/features";
import "./styles/runtime-status.css";

const appIconUrl = new URL("../src-tauri/icons/icon.ico", import.meta.url).href;

type LoadState =
  | { status: "loading" }
  | { status: "ready"; snapshot: RuntimeSnapshot }
  | { status: "error"; message: string };

function RuntimeApp() {
  const [loadState, setLoadState] = useState<LoadState>({ status: "loading" });
  const [listenerWarning, setListenerWarning] = useState<string | null>(null);
  const [refreshWarning, setRefreshWarning] = useState<string | null>(null);
  const latestLaunch = useRef<LaunchRequestedEvent | null>(null);
  const launchSequence = useRef(0);

  useEffect(() => {
    void featuresStore.getState().initialize();
  }, []);

  const refresh = useCallback(async (showLoading = true) => {
    const sequenceAtStart = launchSequence.current;
    if (showLoading) {
      setLoadState({ status: "loading" });
    }
    try {
      const snapshot = await commands.appRuntimeSnapshot();
      const launchDuringRequest =
        launchSequence.current === sequenceAtStart ? null : latestLaunch.current;
      setLoadState({
        status: "ready",
        snapshot: mergeLatestLaunch(snapshot, launchDuringRequest),
      });
      setRefreshWarning(null);
    } catch (error) {
      if (showLoading) {
        setLoadState({ status: "error", message: runtimeErrorMessage(error) });
      } else {
        setRefreshWarning("启动请求已收到，但完整运行状态暂时无法刷新。可稍后重新读取。");
      }
    }
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    const initialize = async () => {
      try {
        const stopListening = await events.appLaunchRequested.listen((event) => {
          latestLaunch.current = event.payload;
          launchSequence.current += 1;
          setLoadState((current) =>
            current.status === "ready"
              ? {
                  status: "ready",
                  snapshot: mergeLatestLaunch(current.snapshot, event.payload),
                }
              : current,
          );
          void refresh(false);
        });
        if (disposed) {
          stopListening();
          return;
        }
        unlisten = stopListening;
      } catch {
        if (!disposed) {
          setListenerWarning("实时启动监听不可用；重新打开窗口可刷新完整状态。");
        }
      }
      if (!disposed) {
        await refresh();
      }
    };

    void initialize();
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [refresh]);

  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="brand-lockup">
          <img className="brand-icon" src={appIconUrl} alt="" width="40" height="40" />
          <div>
            <strong className="brand-name">WubiLex</strong>
            <span className="brand-context">五笔词库工具</span>
          </div>
        </div>
        <span className="stage-badge">S1 Runtime</span>
      </header>

      <main id="main-content" className="runtime-main">
        <div className="page-heading">
          <div>
            <p className="eyebrow">应用外壳</p>
            <h1>运行状态</h1>
          </div>
          {loadState.status === "ready" ? (
            <span className="sync-state" aria-live="polite">
              <span className="status-dot positive" aria-hidden="true" />
              已连接本地运行时
            </span>
          ) : null}
        </div>

        {loadState.status === "loading" ? <LoadingState /> : null}
        {loadState.status === "error" ? (
          <LoadError message={loadState.message} onRetry={() => void refresh()} />
        ) : null}
        {loadState.status === "ready" ? (
          <RuntimeStatus
            snapshot={loadState.snapshot}
            eventWarning={listenerWarning ?? refreshWarning}
          />
        ) : null}
      </main>
    </div>
  );
}

function LoadingState() {
  return (
    <section className="load-state" aria-live="polite" aria-busy="true">
      <span className="loading-indicator" aria-hidden="true" />
      <div>
        <h2>正在读取运行时状态</h2>
        <p>正在检查进程权限、会话标记和启动请求。</p>
      </div>
    </section>
  );
}

function LoadError({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <section className="load-error" role="alert">
      <div>
        <h2>无法连接本地运行时</h2>
        <p>{message}</p>
      </div>
      <button type="button" onClick={onRetry}>
        重新读取
      </button>
    </section>
  );
}

function RuntimeStatus({
  snapshot,
  eventWarning,
}: {
  snapshot: RuntimeSnapshot;
  eventWarning: string | null;
}) {
  const latestLaunch = snapshot.latestSecondaryLaunch ?? snapshot.primaryLaunch;
  const statuses = [
    { label: "进程权限", presentation: describePrivilege(snapshot.privilege) },
    {
      label: "会话检查",
      presentation: describeRecovery(snapshot.previousAbnormalSessionCount),
    },
    { label: "最近启动", presentation: describeLaunch(latestLaunch) },
  ];
  const notices = useMemo(() => collectVisibleNotices(snapshot), [snapshot]);

  return (
    <>
      <section className="status-grid" aria-label="运行时摘要">
        {statuses.map((status) => (
          <StatusCell key={status.label} label={status.label} presentation={status.presentation} />
        ))}
      </section>

      <section className="detail-section" aria-labelledby="launch-heading">
        <div className="section-heading">
          <div>
            <p className="eyebrow">最近请求</p>
            <h2 id="launch-heading">启动参数</h2>
          </div>
          <span className="request-source">
            {snapshot.latestSecondaryLaunch ? "第二实例" : "主实例"}
          </span>
        </div>
        <LaunchDetails launch={latestLaunch} />
      </section>

      <section className="detail-section" aria-labelledby="notice-heading" aria-live="polite">
        <div className="section-heading">
          <div>
            <p className="eyebrow">诊断</p>
            <h2 id="notice-heading">可见警告</h2>
          </div>
          <span className="notice-count">{notices.length + (eventWarning ? 1 : 0)}</span>
        </div>

        {notices.length === 0 && !eventWarning ? (
          <p className="empty-notice">当前没有需要处理的运行时警告。</p>
        ) : (
          <ul className="notice-list">
            {eventWarning ? (
              <li className="notice-row warning">
                <span className="status-dot warning" aria-hidden="true" />
                <div>
                  <strong>实时监听受限</strong>
                  <p>{eventWarning}</p>
                </div>
              </li>
            ) : null}
            {notices.map((notice) => (
              <li key={notice.key} className={`notice-row ${notice.tone}`}>
                <span className={`status-dot ${notice.tone}`} aria-hidden="true" />
                <div>
                  <strong>{notice.summary}</strong>
                  {notice.detail ? <p>{notice.detail}</p> : null}
                </div>
              </li>
            ))}
          </ul>
        )}
      </section>
    </>
  );
}

function StatusCell({
  label,
  presentation,
}: {
  label: string;
  presentation: StatusPresentation;
}) {
  return (
    <div className="status-cell">
      <span className="status-label">{label}</span>
      <div className="status-title">
        <span className={`status-dot ${presentation.tone}`} aria-hidden="true" />
        <strong>{presentation.label}</strong>
      </div>
      <p>{presentation.detail}</p>
    </div>
  );
}

function LaunchDetails({ launch }: { launch: LaunchRequestedEvent }) {
  return (
    <dl className="launch-details">
      <div>
        <dt>窗口模式</dt>
        <dd>{launch.request.startHidden ? "隐藏" : "可见"}</dd>
      </div>
      <div>
        <dt>内部导航</dt>
        <dd>
          {launch.request.navigationPath ? (
            <code>{launch.request.navigationPath}</code>
          ) : (
            "未指定"
          )}
        </dd>
      </div>
      <div>
        <dt>参数警告</dt>
        <dd>{launch.notices.length}</dd>
      </div>
    </dl>
  );
}

const root = document.getElementById("root");
if (!root) {
  throw new Error("WubiLex root element is missing");
}

createRoot(root).render(
  <StrictMode>
    <RuntimeApp />
  </StrictMode>,
);
