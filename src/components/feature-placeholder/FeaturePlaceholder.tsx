import type { TargetMilestone } from "../../types/generated/bindings";
import { Construction } from "../../icons/ui";
import { useTranslation } from "react-i18next";

export type FeaturePlaceholderVariant = "page" | "section" | "inline";

interface FeaturePlaceholderProps {
  variant: FeaturePlaceholderVariant;
  title: string;
  description: string;
  milestone?: TargetMilestone;
}

export function FeaturePlaceholder({
  variant,
  title,
  description,
  milestone,
}: FeaturePlaceholderProps) {
  const { t } = useTranslation("shell");
  return (
    <div
      className={`feature-placeholder feature-placeholder-${variant}`}
      data-placeholder-variant={variant}
      role="status"
    >
      <Construction aria-hidden="true" strokeWidth={1.8} />
      <div className="feature-placeholder-copy">
        <p className="feature-placeholder-state">{t("placeholder.state")}</p>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      <span className="feature-placeholder-stage">
        {milestone
          ? t("placeholder.milestone", { milestone: milestone.toUpperCase() })
          : t("placeholder.planned")}
      </span>
    </div>
  );
}
