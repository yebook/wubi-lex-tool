import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { RouteHeading } from "../../app/layout/route-heading";
import { useAppRuntime } from "../../app/providers/app-runtime-provider";
import { useUiPreferences } from "../../app/providers/ui-preferences-provider";
import { useAppWindowControls } from "../../app/providers/window-controls-provider";
import { useAppNavigation } from "../../app/router/navigation-provider";
import { Button } from "../../components/ui";
import {
  collectVisibleNotices,
  describeLaunch,
  describePrivilege,
  describeRecovery,
  mergeRuntimeNotices,
} from "../../runtime-view";
import type { StatusPresentation } from "../../runtime-view";
import type {
  LaunchRequestedEvent,
  RuntimeSnapshot,
} from "../../types/generated/bindings";

export function OverviewRoute() {
  const { t } = useTranslation("runtime");
  const runtime = useAppRuntime();
  const preferences = useUiPreferences();
  const windowControls = useAppWindowControls();
  const navigation = useAppNavigation();
  const warnings = [
    navigation.warning,
    preferences.warning,
    runtime.listenerWarning,
    runtime.refreshWarning,
    windowControls.warning,
  ].filter((warning): warning is string => Boolean(warning));

  return (
    <div className="route-page overview-page">
      <RouteHeading eyebrow={t("eyebrow")} title={t("title")} />
      {runtime.loadState.status === "loading" ? <LoadingState /> : null}
      {runtime.loadState.status === "error" ? (
        <LoadError
          message={runtime.loadState.message}
          onRetry={() => void runtime.refresh()}
        />
      ) : null}
      {runtime.loadState.status === "ready" ? (
        <RuntimeStatus
          snapshot={runtime.loadState.snapshot}
          nativeNotices={windowControls.notices}
          warnings={warnings}
        />
      ) : null}
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
  warnings,
}: {
  snapshot: RuntimeSnapshot;
  nativeNotices: RuntimeSnapshot["notices"];
  warnings: string[];
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
            {notices.length + warnings.length}
          </span>
        </div>
        {notices.length === 0 && warnings.length === 0 ? (
          <p className="empty-notice">{t("notices.empty")}</p>
        ) : (
          <ul className="notice-list">
            {warnings.map((warning, index) => (
              <li key={`frontend-${index}`} className="notice-row warning">
                <span className="status-dot warning" aria-hidden="true" />
                <div>
                  <strong>{t("warning.restricted")}</strong>
                  <p>{warning}</p>
                </div>
              </li>
            ))}
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
