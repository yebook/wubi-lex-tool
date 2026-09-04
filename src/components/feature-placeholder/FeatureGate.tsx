import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { useStore } from "zustand";

import { Button } from "../ui";
import { RefreshCw } from "../../icons/ui";
import {
  featuresStore,
  selectFeature,
  selectFeatureError,
  selectFeatureStatus,
} from "../../stores/features";
import type { AppFeatureId } from "../../types/generated/bindings";
import { FeaturePlaceholder } from "./FeaturePlaceholder";
import type { FeaturePlaceholderVariant } from "./FeaturePlaceholder";

interface FeatureGateProps {
  feature: AppFeatureId;
  variant: FeaturePlaceholderVariant;
  title: string;
  description: string;
  children: ReactNode;
}

export function FeatureGate({
  feature,
  variant,
  title,
  description,
  children,
}: FeatureGateProps) {
  const { t } = useTranslation(["shell", "common"]);
  const status = useStore(featuresStore, selectFeatureStatus);
  const error = useStore(featuresStore, selectFeatureError);
  const record = useStore(featuresStore, selectFeature(feature));
  const retry = useStore(featuresStore, (state) => state.retry);

  if (status === "loading") {
    return (
      <div
        className={`feature-gate-state feature-gate-${variant}`}
        role="status"
        aria-busy="true"
      >
        <span className="loading-indicator" aria-hidden="true" />
        <div>
          <h2>{t("shell:featureGate.loadingTitle")}</h2>
          <p>{t("shell:featureGate.loadingDetail")}</p>
        </div>
      </div>
    );
  }

  if (status === "failed" || !record) {
    return (
      <div
        className={`feature-gate-state feature-gate-${variant}`}
        role="alert"
      >
        <div>
          <h2>{t("shell:featureGate.errorTitle")}</h2>
          <p>
            {error?.message ??
              t("shell:featureGate.missingFeature", { feature })}
          </p>
        </div>
        <Button variant="outline" onClick={() => void retry()}>
          <RefreshCw aria-hidden="true" strokeWidth={1.8} />
          {t("common:retry")}
        </Button>
      </div>
    );
  }

  if (!record.available) {
    return (
      <FeaturePlaceholder
        variant={variant}
        title={title}
        description={description}
        milestone={record.targetMilestone}
      />
    );
  }

  return children;
}
