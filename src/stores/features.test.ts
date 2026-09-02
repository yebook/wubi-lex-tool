import { describe, expect, it } from "vitest";

import type { AppFeatureCatalog } from "../types/generated/bindings";
import type { FeatureClient } from "../lib/features-client";
import {
  createFeatureStore,
  selectFeature,
  selectFeatureAvailable,
} from "./features";

const disabledCatalog: AppFeatureCatalog = {
  features: [
    {
      id: "lexiconRead",
      available: false,
      targetMilestone: "s2",
      unavailableReason: "notIncludedInBuild",
    },
    {
      id: "systemWrite",
      available: false,
      targetMilestone: "s3",
      unavailableReason: "notIncludedInBuild",
    },
  ],
};

const enabledCatalog: AppFeatureCatalog = {
  features: [
    {
      id: "lexiconRead",
      available: true,
      targetMilestone: "s2",
      unavailableReason: null,
    },
  ],
};

describe("feature store", () => {
  it("deduplicates concurrent StrictMode-style initialization", async () => {
    let resolve: ((catalog: AppFeatureCatalog) => void) | undefined;
    let calls = 0;
    const client: FeatureClient = {
      fetchCatalog: () => {
        calls += 1;
        return new Promise((complete) => {
          resolve = complete;
        });
      },
    };
    const store = createFeatureStore(client);

    const first = store.getState().initialize();
    const second = store.getState().initialize();
    expect(first).toBe(second);
    expect(calls).toBe(1);
    resolve?.(disabledCatalog);
    await first;

    expect(store.getState().status).toBe("ready");
    expect(store.getState().catalog).toEqual(disabledCatalog);
  });

  it("keeps failure visible and starts a new request on retry", async () => {
    let calls = 0;
    const client: FeatureClient = {
      fetchCatalog: () => {
        calls += 1;
        return calls === 1
          ? Promise.reject(new Error("IPC unavailable"))
          : Promise.resolve(enabledCatalog);
      },
    };
    const store = createFeatureStore(client);

    await store.getState().initialize();
    expect(store.getState()).toMatchObject({
      status: "failed",
      error: { message: "IPC unavailable" },
    });
    await store.getState().retry();
    expect(calls).toBe(2);
    expect(store.getState()).toMatchObject({ status: "ready", error: null });
  });

  it("provides typed lookups and full replacement removes stale records", () => {
    const store = createFeatureStore({
      fetchCatalog: () => Promise.resolve(disabledCatalog),
    });
    store.getState().replace(disabledCatalog);

    expect(store.getState().feature("lexiconRead")?.available).toBe(false);
    expect(store.getState().isAvailable("lexiconRead")).toBe(false);
    expect(
      selectFeature("systemWrite")(store.getState())?.targetMilestone,
    ).toBe("s3");

    store.getState().replace(enabledCatalog);
    expect(store.getState().catalog.features).toHaveLength(1);
    expect(selectFeature("systemWrite")(store.getState())).toBeUndefined();
    expect(selectFeatureAvailable("lexiconRead")(store.getState())).toBe(true);
  });
});
