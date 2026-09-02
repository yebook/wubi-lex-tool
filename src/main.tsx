import {
  StrictMode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { createRoot } from "react-dom/client";
import { I18nextProvider, useTranslation } from "react-i18next";
import packageMetadata from "../package.json";

import {
  UiPreferencesProvider,
  useUiPreferences,
} from "./app/providers/ui-preferences-provider";
import { Button, OverlayProvider } from "./components/ui";
import { WindowTitleBar } from "./components/window-title-bar/WindowTitleBar";
import { useWindowControls } from "./hooks/use-window-controls";
import { i18n } from "./i18n";
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
  mergeRuntimeNotices,
  runtimeErrorMessage,
} from "./runtime-view";
import type { StatusPresentation } from "./runtime-view";
import { featuresStore } from "./stores/features";
import "./styles/theme.css";
import "./styles/runtime-status.css";

const appIconUrl = new URL("../src-tauri/icons/icon.ico", import.meta.url).href;

type LoadState =
  | { status: "loading" }
  | { status: "ready"; snapshot: RuntimeSnapshot }
  | { status: "error"; message: string };

function RuntimeApp() {
  const { t } = useTranslation("runtime");
  const uiPreferences = useUiPreferences();
  const [loadState, setLoadState] = useState<LoadState>({ status: "loading" });
  const [listenerWarning, setListenerWarning] = useState<string | null>(null);
  const [refreshWarning, setRefreshWarning] = useState<string | null>(null);
  const latestLaunch = useRef<LaunchRequestedEvent | null>(null);
  const launchSequence = useRef(0);
  const windowControls = useWindowControls();

  useEffect(() => {
    void featuresStore.getState().initialize();
  }, []);

  const refresh = useCallback(
    async (showLoading = true) => {
      const sequenceAtStart = launchSequence.current;
      if (showLoading) {
        setLoadState({ status: "loading" });
      }
      try {
        const snapshot = await commands.appRuntimeSnapshot();
        const launchDuringRequest =
          launchSequence.current === sequenceAtStart
            ? null
            : latestLaunch.current;
        setLoadState({
          status: "ready",
          snapshot: mergeLatestLaunch(snapshot, launchDuringRequest),
        });
        setRefreshWarning(null);
      } catch (error) {
        if (showLoading) {
          setLoadState({
            status: "error",
            message: runtimeErrorMessage(error, t),
          });
        } else {
          setRefreshWarning(t("warning.refresh"));
        }
      }
    },
    [t],
  );

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    const initialize = async () => {
      try {
        const stopListening = await events.appLaunchRequested.listen(
          (event) => {
            latestLaunch.current = event.payload;
            launchSequence.current += 1;
            setLoadState((current) =>
              current.status === "ready"
                ? {
                    status: "ready",
                    snapshot: mergeLatestLaunch(
                      current.snapshot,
                      event.payload,
                    ),
                  }
                : current,
            );
            void refresh(false);
          },
        );
        if (disposed) {
          stopListening();
          return;
        }
        unlisten = stopListening;
      } catch {
        if (!disposed) {
          setListenerWarning(t("warning.listener"));
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
  }, [refresh, t]);

  return (
    <div className="app-shell">
      <WindowTitleBar
        iconUrl={appIconUrl}
        version={packageMetadata.version}
        snapshot={windowControls.snapshot}
        onControl={(intent) => void windowControls.control(intent)}
      />

      <main id="main-content" className="runtime-main">
        <div className="page-heading">
          <div>
            <p className="eyebrow">{t("eyebrow")}</p>
            <h1>{t("title")}</h1>
          </div>
          {loadState.status === "ready" ? (
            <span className="sync-state" aria-live="polite">
              <span className="status-dot positive" aria-hidden="true" />
              {t("connected")}
            </span>
          ) : null}
        </div>

        {loadState.status === "loading" ? <LoadingState /> : null}
        {loadState.status === "error" ? (
          <LoadError
            message={loadState.message}
            onRetry={() => void refresh()}
          />
        ) : null}
        {loadState.status === "ready" ? (
          <RuntimeStatus
            snapshot={loadState.snapshot}
            nativeNotices={windowControls.notices}
            eventWarning={
              uiPreferences.warning ??
              listenerWarning ??
              refreshWarning ??
              windowControls.warning
            }
          />
        ) : null}
      </main>
    </div>
  );
}

function LoadingState() {
  const { t } = useTranslation("runtime");
  return (
    <section className="load-state" aria-live="polite" aria-busy="true">
      <span className="loading-indicator" aria-hidden="true" />
      <div>
        <h2>{t("loading.title")}</h2>
        <p>{t("loading.detail")}</p>
      </div>
    </section>
  );
}

function LoadError({
  message,
  onRetry,
}: {
  message: string;
  onRetry: () => void;
}) {
  const { t } = useTranslation(["runtime", "common"]);
  return (
    <section className="load-error" role="alert">
      <div>
        <h2>{t("runtime:loadError.title")}</h2>
        <p>{message}</p>
      </div>
      <Button onClick={onRetry}>{t("common:retry")}</Button>
    </section>
  );
}

function RuntimeStatus({
  snapshot,
  nativeNotices,
  eventWarning,
}: {
  snapshot: RuntimeSnapshot;
  nativeNotices: RuntimeSnapshot["notices"];
  eventWarning: string | null;
}) {
  const { t } = useTranslation("runtime");
  const latestLaunch = snapshot.latestSecondaryLaunch ?? snapshot.primaryLaunch;
  const statuses = [
    {
      label: t("summary.privilege"),
      presentation: describePrivilege(snapshot.privilege, t),
    },
    {
      label: t("summary.recovery"),
      presentation: describeRecovery(snapshot.previousAbnormalSessionCount, t),
    },
    {
      label: t("summary.launch"),
      presentation: describeLaunch(latestLaunch, t),
    },
  ];
  const notices = useMemo(
    () => collectVisibleNotices(mergeRuntimeNotices(snapshot, nativeNotices)),
    [nativeNotices, snapshot],
  );

  return (
    <>
      <section className="status-grid" aria-label={t("summary.label")}>
        {statuses.map((status) => (
          <StatusCell
            key={status.label}
            label={status.label}
            presentation={status.presentation}
          />
        ))}
      </section>

      <section className="detail-section" aria-labelledby="launch-heading">
        <div className="section-heading">
          <div>
            <p className="eyebrow">{t("request.eyebrow")}</p>
            <h2 id="launch-heading">{t("request.title")}</h2>
          </div>
          <span className="request-source">
            {snapshot.latestSecondaryLaunch
              ? t("request.secondary")
              : t("request.primary")}
          </span>
        </div>
        <LaunchDetails launch={latestLaunch} />
      </section>

      <section
        className="detail-section"
        aria-labelledby="notice-heading"
        aria-live="polite"
      >
        <div className="section-heading">
          <div>
            <p className="eyebrow">{t("notices.eyebrow")}</p>
            <h2 id="notice-heading">{t("notices.title")}</h2>
          </div>
          <span className="notice-count">
            {notices.length + (eventWarning ? 1 : 0)}
          </span>
        </div>

        {notices.length === 0 && !eventWarning ? (
          <p className="empty-notice">{t("notices.empty")}</p>
        ) : (
          <ul className="notice-list">
            {eventWarning ? (
              <li className="notice-row warning">
                <span className="status-dot warning" aria-hidden="true" />
                <div>
                  <strong>{t("warning.restricted")}</strong>
                  <p>{eventWarning}</p>
                </div>
              </li>
            ) : null}
            {notices.map((notice) => (
              <li key={notice.key} className={`notice-row ${notice.tone}`}>
                <span
                  className={`status-dot ${notice.tone}`}
                  aria-hidden="true"
                />
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
        <span
          className={`status-dot ${presentation.tone}`}
          aria-hidden="true"
        />
        <strong>{presentation.label}</strong>
      </div>
      <p>{presentation.detail}</p>
    </div>
  );
}

function LaunchDetails({ launch }: { launch: LaunchRequestedEvent }) {
  const { t } = useTranslation("runtime");
  return (
    <dl className="launch-details">
      <div>
        <dt>{t("request.windowMode")}</dt>
        <dd>
          {launch.request.startHidden
            ? t("request.hidden")
            : t("request.visible")}
        </dd>
      </div>
      <div>
        <dt>{t("request.navigation")}</dt>
        <dd>
          {launch.request.navigationPath ? (
            <code>{launch.request.navigationPath}</code>
          ) : (
            t("request.unspecified")
          )}
        </dd>
      </div>
      <div>
        <dt>{t("request.warnings")}</dt>
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
    <I18nextProvider i18n={i18n}>
      <UiPreferencesProvider>
        <OverlayProvider>
          <RuntimeApp />
        </OverlayProvider>
      </UiPreferencesProvider>
    </I18nextProvider>
  </StrictMode>,
);
