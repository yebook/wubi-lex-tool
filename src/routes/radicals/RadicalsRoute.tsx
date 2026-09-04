import { useTranslation } from "react-i18next";

import { RouteHeading } from "../../app/layout/route-heading";
import { FeatureGate } from "../../components/feature-placeholder";

export function RadicalsRoute() {
  const { t } = useTranslation("shell");
  return (
    <div className="route-page">
      <RouteHeading
        eyebrow={t("routes.radicals")}
        title={t("radicals.title")}
        detail={t("radicals.detail")}
      />
      <FeatureGate
        feature="radicalReference"
        variant="page"
        title={t("radicals.placeholderTitle")}
        description={t("radicals.placeholderDetail")}
      >
        <section className="domain-ready">
          <h2>{t("radicals.readyTitle")}</h2>
          <p>{t("radicals.readyDetail")}</p>
        </section>
      </FeatureGate>
    </div>
  );
}
