import { createStore } from "zustand/vanilla";

import { i18n } from "../i18n";
import type {
  AppFeature,
  AppFeatureCatalog,
  AppFeatureId,
} from "../types/generated/bindings";
import { featureClient } from "../lib/features-client";
import type { FeatureClient } from "../lib/features-client";

export type FeatureLoadStatus = "loading" | "ready" | "failed";

export interface FeatureClientFailure {
  message: string;
}

export interface FeaturesState {
  status: FeatureLoadStatus;
  catalog: AppFeatureCatalog;
  error: FeatureClientFailure | null;
  initialize(): Promise<void>;
  retry(): Promise<void>;
  replace(catalog: AppFeatureCatalog): void;
  feature(id: AppFeatureId): AppFeature | undefined;
  isAvailable(id: AppFeatureId): boolean;
}

const emptyCatalog: AppFeatureCatalog = { features: [] };

export function createFeatureStore(client: FeatureClient) {
  let inFlight: Promise<void> | null = null;

  return createStore<FeaturesState>((set, get) => {
    const load = (force: boolean): Promise<void> => {
      if (inFlight) {
        return inFlight;
      }
      if (!force && get().status === "ready") {
        return Promise.resolve();
      }

      set({ status: "loading", error: null });
      const request = client
        .fetchCatalog()
        .then((catalog) => {
          set({ status: "ready", catalog, error: null });
        })
        .catch((error: unknown) => {
          set({ status: "failed", error: invocationFailure(error) });
        })
        .finally(() => {
          if (inFlight === request) {
            inFlight = null;
          }
        });
      inFlight = request;
      return request;
    };

    return {
      status: "loading",
      catalog: emptyCatalog,
      error: null,
      initialize: () => load(false),
      retry: () => load(true),
      replace: (catalog) => set({ status: "ready", catalog, error: null }),
      feature: (id) =>
        get().catalog.features.find((feature) => feature.id === id),
      isAvailable: (id) =>
        get().catalog.features.some(
          (feature) => feature.id === id && feature.available,
        ),
    };
  });
}

export const featuresStore = createFeatureStore(featureClient);

export const selectFeatureStatus = (state: FeaturesState) => state.status;
export const selectFeatureError = (state: FeaturesState) => state.error;
export const selectFeature = (id: AppFeatureId) => (state: FeaturesState) =>
  state.catalog.features.find((feature) => feature.id === id);
export const selectFeatureAvailable =
  (id: AppFeatureId) => (state: FeaturesState) =>
    state.catalog.features.some(
      (feature) => feature.id === id && feature.available,
    );

function invocationFailure(error: unknown): FeatureClientFailure {
  const message =
    error instanceof Error && error.message
      ? error.message
      : i18n.t("runtime:featureCatalogFallback");
  return { message: [...message].slice(0, 512).join("") };
}
