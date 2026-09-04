import { useTranslation } from "react-i18next";

import { RouteHeading } from "../../app/layout/route-heading";
import { FeatureGate } from "../../components/feature-placeholder";

export function LearningRoute() {
  const { t } = useTranslation("shell");
  return (
    <div className="route-page">
      <RouteHeading
        eyebrow={t("routes.learning")}
        title={t("learning.title")}
        detail={t("learning.detail")}
      />
      <FeatureGate
        feature="selfLearning"
        variant="page"
        title={t("learning.placeholderTitle")}
        description={t("learning.placeholderDetail")}
      >
        <section className="domain-ready">
          <h2>{t("learning.readyTitle")}</h2>
          <p>{t("learning.readyDetail")}</p>
        </section>
      </FeatureGate>
    </div>
  );
}
