import { useTranslation } from "react-i18next";

import { RouteHeading } from "../../app/layout/route-heading";
import { FeatureGate } from "../../components/feature-placeholder";

export function LookupRoute() {
  const { t } = useTranslation("shell");
  return (
    <div className="route-page">
      <RouteHeading
        eyebrow={t("routes.lookup")}
        title={t("lookup.title")}
        detail={t("lookup.detail")}
      />
      <FeatureGate
        feature="reverseLookup"
        variant="page"
        title={t("lookup.placeholderTitle")}
        description={t("lookup.placeholderDetail")}
      >
        <section className="domain-ready">
          <h2>{t("lookup.readyTitle")}</h2>
          <p>{t("lookup.readyDetail")}</p>
        </section>
      </FeatureGate>
    </div>
  );
}
