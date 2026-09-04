import { describe, expect, it } from "vitest";

import { routeCatalog } from "./catalog";
import { validateProductPath } from "./path";

describe("route catalog", () => {
  it("freezes the seven routes, order, and generated feature mapping", () => {
    expect(routeCatalog.map(({ id, path }) => ({ id, path }))).toEqual([
      { id: "overview", path: "/overview" },
      { id: "lexicons", path: "/lexicons" },
      { id: "phrases", path: "/phrases" },
      { id: "lookup", path: "/lookup" },
      { id: "radicals", path: "/radicals" },
      { id: "learning", path: "/learning" },
      { id: "settings", path: "/settings" },
    ]);
    expect(new Set(routeCatalog.map((route) => route.id)).size).toBe(7);
    expect(new Set(routeCatalog.map((route) => route.path)).size).toBe(7);
    expect(
      routeCatalog.map((route) =>
        "feature" in route ? route.feature : undefined,
      ),
    ).toEqual([
      undefined,
      "lexiconRead",
      "phraseRead",
      "reverseLookup",
      "radicalReference",
      "selfLearning",
      undefined,
    ]);
  });
});

describe("validateProductPath", () => {
  it("accepts exact catalog paths and normalizes the root alias", () => {
    for (const route of routeCatalog) {
      expect(validateProductPath(route.path)).toEqual({
        kind: "canonical",
        path: route.path,
        warning: null,
      });
    }
    expect(validateProductPath("/")).toEqual({
      kind: "redirect",
      path: "/overview",
      warning: null,
    });
  });

  it.each([
    "/Overview",
    "/overview/",
    "/settings/runtime",
    "/lookup?q=a",
    "/lookup#x",
  ])("fails closed for non-canonical path %s", (path) => {
    const result = validateProductPath(path);
    expect(result.kind).toBe("redirect");
    expect(result.path).toBe("/overview");
    expect(result.warning).toContain(path);
    expect([...(result.warning ?? "")].length).toBeLessThanOrEqual(512);
  });

  it("bounds an unknown path warning", () => {
    const result = validateProductPath(`/${"路".repeat(900)}`);
    expect([...(result.warning ?? "")].length).toBeLessThanOrEqual(512);
  });
});
