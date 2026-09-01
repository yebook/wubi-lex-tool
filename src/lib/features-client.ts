import type { AppFeatureCatalog } from "../types/generated/bindings";
import { commands } from "../types/generated/bindings";

export interface FeatureClient {
  fetchCatalog(): Promise<AppFeatureCatalog>;
}

export const featureClient: FeatureClient = {
  fetchCatalog: () => commands.appFeatures(),
};
