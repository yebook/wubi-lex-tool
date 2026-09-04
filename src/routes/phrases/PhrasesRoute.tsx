import { useTranslation } from "react-i18next";

import { RouteHeading } from "../../app/layout/route-heading";
import { FeatureGate } from "../../components/feature-placeholder";

export function PhrasesRoute() {
  const { t } = useTranslation("shell");
  return (
    <div className="route-page">
      <RouteHeading
        eyebrow={t("routes.phrases")}
        title={t("phrases.title")}
        detail={t("phrases.detail")}
      />
      <FeatureGate
        feature="phraseRead"
        variant="page"
        title={t("phrases.placeholderTitle")}
        description={t("phrases.placeholderDetail")}
      >
        <section className="domain-ready">
          <h2>{t("phrases.readyTitle")}</h2>
          <p>{t("phrases.readyDetail")}</p>
        </section>
      </FeatureGate>
    </div>
  );
}
