import { useTranslation } from "react-i18next";

import { RouteHeading } from "../../app/layout/route-heading";
import { FeatureGate } from "../../components/feature-placeholder";

export function LexiconsRoute() {
  const { t } = useTranslation("shell");
  return (
    <div className="route-page">
      <RouteHeading
        eyebrow={t("routes.lexicons")}
        title={t("lexicons.title")}
        detail={t("lexicons.detail")}
      />
      <FeatureGate
        feature="lexiconRead"
        variant="page"
        title={t("lexicons.placeholderTitle")}
        description={t("lexicons.placeholderDetail")}
      >
        <DomainReady
          title={t("lexicons.readyTitle")}
          detail={t("lexicons.readyDetail")}
        />
      </FeatureGate>
    </div>
  );
}

function DomainReady({ title, detail }: { title: string; detail: string }) {
  return (
    <section className="domain-ready">
      <h2>{title}</h2>
      <p>{detail}</p>
    </section>
  );
}
