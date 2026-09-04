import { useTranslation } from "react-i18next";

import { RouteHeading } from "../../app/layout/route-heading";
import { useUiPreferences } from "../../app/providers/ui-preferences-provider";
import { FeaturePlaceholder } from "../../components/feature-placeholder";
import type { TargetMilestone } from "../../types/generated/bindings";

export function SettingsRoute() {
  const { t } = useTranslation("shell");
  const preferences = useUiPreferences();

  return (
    <div className="route-page settings-page">
      <RouteHeading
        eyebrow={t("routes.settings")}
        title={t("settings.title")}
        detail={t("settings.detail")}
      />
      <DeferredSetting name="ime" milestone="s3" />
      <DeferredSetting name="wubi" milestone="s2" />
      <DeferredSetting name="candidate" milestone="s5" />
      <DeferredSetting name="shortcuts" />

      <section
        className="settings-section"
        aria-labelledby="appearance-heading"
      >
        <div className="settings-section-heading">
          <h2 id="appearance-heading">
            {t("settings.groups.appearance.title")}
          </h2>
          <p>{t("settings.groups.appearance.detail")}</p>
        </div>
        <fieldset
          className="settings-fieldset"
          disabled={preferences.status !== "ready"}
        >
          <legend>{t("settings.appearance.theme")}</legend>
          <div className="segmented-control">
            {(["system", "light", "dark"] as const).map((theme) => (
              <label key={theme}>
                <input
                  type="radio"
                  name="theme"
                  value={theme}
                  checked={preferences.ui.theme === theme}
                  onChange={() => void preferences.setTheme(theme)}
                />
                <span>{t(`settings.appearance.themeOptions.${theme}`)}</span>
              </label>
            ))}
          </div>
        </fieldset>
        <fieldset
          className="settings-fieldset"
          disabled={preferences.status !== "ready"}
        >
          <legend>{t("settings.appearance.density")}</legend>
          <div className="segmented-control">
            {(["standard", "compact"] as const).map((density) => (
              <label key={density}>
                <input
                  type="radio"
                  name="density"
                  value={density}
                  checked={preferences.ui.density === density}
                  onChange={() => void preferences.setDensity(density)}
                />
                <span>
                  {t(`settings.appearance.densityOptions.${density}`)}
                </span>
              </label>
            ))}
          </div>
        </fieldset>
        <label className="settings-toggle">
          <span>
            <strong>{t("settings.appearance.sidebar")}</strong>
            <small>{t("settings.appearance.sidebarDetail")}</small>
          </span>
          <input
            type="checkbox"
            checked={preferences.ui.sidebarCollapsed}
            disabled={preferences.status !== "ready"}
            onChange={(event) =>
              void preferences.setSidebarCollapsed(event.currentTarget.checked)
            }
          />
        </label>
        {preferences.warning ? (
          <p className="settings-warning" role="alert">
            {preferences.warning}
          </p>
        ) : null}
      </section>

      <DeferredSetting name="network" milestone="s6" />
      <DeferredSetting name="data" milestone="s6" />
      <DeferredSetting name="about" />
    </div>
  );
}

function DeferredSetting({
  name,
  milestone,
}: {
  name:
    "ime" | "wubi" | "candidate" | "shortcuts" | "network" | "data" | "about";
  milestone?: TargetMilestone;
}) {
  const { t } = useTranslation("shell");
  return (
    <section className="settings-section">
      <FeaturePlaceholder
        variant="section"
        title={t(`settings.groups.${name}.title`)}
        description={t(`settings.groups.${name}.detail`)}
        {...(milestone ? { milestone } : {})}
      />
    </section>
  );
}
