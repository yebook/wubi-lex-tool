//! Stable application capability catalog projected from Cargo features.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Stable identifiers consumed by routes, actions, and settings.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AppFeatureId {
    LexiconRead,
    PhraseRead,
    ReverseLookup,
    SystemWrite,
    LexiconEdit,
    PhraseEdit,
    RadicalReference,
    ResourceSync,
    SystemSettings,
    ResourceUpdate,
    LegacyMigration,
    SelfLearning,
}

/// Milestone that owns a capability implementation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum TargetMilestone {
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
}

/// Typed reason a catalog entry is not available in the current build.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FeatureUnavailableReason {
    NotIncludedInBuild,
}

/// One stable capability record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppFeature {
    pub id: AppFeatureId,
    pub available: bool,
    pub target_milestone: TargetMilestone,
    pub unavailable_reason: Option<FeatureUnavailableReason>,
}

/// Complete feature snapshot returned at frontend startup.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppFeatureCatalog {
    pub features: Vec<AppFeature>,
}

const FEATURE_DEFINITIONS: [(AppFeatureId, bool, TargetMilestone); 12] = [
    (
        AppFeatureId::LexiconRead,
        cfg!(feature = "feat-m1-read"),
        TargetMilestone::S2,
    ),
    (
        AppFeatureId::PhraseRead,
        cfg!(feature = "feat-m2-read"),
        TargetMilestone::S2,
    ),
    (
        AppFeatureId::ReverseLookup,
        cfg!(feature = "feat-m3-lookup"),
        TargetMilestone::S2,
    ),
    (
        AppFeatureId::SystemWrite,
        cfg!(feature = "feat-m4-system-write"),
        TargetMilestone::S3,
    ),
    (
        AppFeatureId::LexiconEdit,
        cfg!(feature = "feat-m1-edit"),
        TargetMilestone::S4,
    ),
    (
        AppFeatureId::PhraseEdit,
        cfg!(feature = "feat-m2-edit"),
        TargetMilestone::S4,
    ),
    (
        AppFeatureId::RadicalReference,
        cfg!(feature = "feat-m5-radicals"),
        TargetMilestone::S5,
    ),
    (
        AppFeatureId::ResourceSync,
        cfg!(feature = "feat-m6-resource-sync"),
        TargetMilestone::S5,
    ),
    (
        AppFeatureId::SystemSettings,
        cfg!(feature = "feat-m4-settings"),
        TargetMilestone::S6,
    ),
    (
        AppFeatureId::ResourceUpdate,
        cfg!(feature = "feat-m6-update"),
        TargetMilestone::S6,
    ),
    (
        AppFeatureId::LegacyMigration,
        cfg!(feature = "feat-m7-legacy-migration"),
        TargetMilestone::S7,
    ),
    (
        AppFeatureId::SelfLearning,
        cfg!(feature = "feat-m8-learning"),
        TargetMilestone::S8,
    ),
];

/// Returns the complete, stable catalog in product order.
pub fn catalog() -> AppFeatureCatalog {
    AppFeatureCatalog {
        features: FEATURE_DEFINITIONS
            .into_iter()
            .map(|(id, available, target_milestone)| AppFeature {
                id,
                available,
                target_milestone,
                unavailable_reason: (!available)
                    .then_some(FeatureUnavailableReason::NotIncludedInBuild),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{AppFeatureId, FeatureUnavailableReason, TargetMilestone, catalog};

    #[test]
    fn catalog_is_complete_stable_and_cfg_projected() {
        let catalog = catalog();
        assert_eq!(
            catalog
                .features
                .iter()
                .map(|feature| (feature.id, feature.target_milestone))
                .collect::<Vec<_>>(),
            vec![
                (AppFeatureId::LexiconRead, TargetMilestone::S2),
                (AppFeatureId::PhraseRead, TargetMilestone::S2),
                (AppFeatureId::ReverseLookup, TargetMilestone::S2),
                (AppFeatureId::SystemWrite, TargetMilestone::S3),
                (AppFeatureId::LexiconEdit, TargetMilestone::S4),
                (AppFeatureId::PhraseEdit, TargetMilestone::S4),
                (AppFeatureId::RadicalReference, TargetMilestone::S5),
                (AppFeatureId::ResourceSync, TargetMilestone::S5),
                (AppFeatureId::SystemSettings, TargetMilestone::S6),
                (AppFeatureId::ResourceUpdate, TargetMilestone::S6),
                (AppFeatureId::LegacyMigration, TargetMilestone::S7),
                (AppFeatureId::SelfLearning, TargetMilestone::S8),
            ]
        );
        for feature in catalog.features {
            assert_eq!(
                feature.unavailable_reason,
                (!feature.available).then_some(FeatureUnavailableReason::NotIncludedInBuild)
            );
        }
    }

    #[test]
    fn serialized_catalog_exposes_product_ids_without_cargo_names() {
        let json = serde_json::to_string(&catalog()).expect("catalog JSON");
        assert!(json.contains("lexiconRead"));
        assert!(json.contains("selfLearning"));
        assert!(!json.contains("feat-m"));
    }

    #[cfg(not(any(
        feature = "feat-m1-read",
        feature = "feat-m1-edit",
        feature = "feat-m2-read",
        feature = "feat-m2-edit",
        feature = "feat-m3-lookup",
        feature = "feat-m4-system-write",
        feature = "feat-m4-settings",
        feature = "feat-m5-radicals",
        feature = "feat-m6-resource-sync",
        feature = "feat-m6-update",
        feature = "feat-m7-legacy-migration",
        feature = "feat-m8-learning"
    )))]
    #[test]
    fn default_build_leaves_every_future_capability_unavailable() {
        assert!(catalog().features.iter().all(|feature| !feature.available));
    }
}
