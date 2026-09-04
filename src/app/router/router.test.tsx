// @vitest-environment jsdom

import { describe, expect, it } from "vitest";

import type { InitialNavigation } from "../providers/app-runtime-provider";
import { routeCatalog } from "./catalog";
import {
  createMemoryAppRouter,
  createRouteObjects,
  replaceInitialHash,
} from "./router";

const initial: InitialNavigation = {
  path: "/overview",
  warning: null,
  consumedLaunchSequence: 0,
};

describe("router factories", () => {
  it("uses one route object tree for the seven paths, alias, and wildcard", () => {
    const routes = createRouteObjects(initial);
    expect(routes).toHaveLength(1);
    expect(routes[0]?.children?.map((route) => route.path)).toEqual([
      "/",
      ...routeCatalog.map((route) => route.path),
      "*",
    ]);
    const memory = createMemoryAppRouter(initial);
    expect(memory.routes[0]?.children?.map((route) => route.path)).toEqual(
      routes[0]?.children?.map((route) => route.path),
    );
  });

  it("replaces the initial hash without adding a history entry", () => {
    window.history.replaceState({ seed: true }, "", "/desktop?mode=test#/old");
    const length = window.history.length;
    replaceInitialHash("/settings");
    expect(window.location.pathname).toBe("/desktop");
    expect(window.location.search).toBe("?mode=test");
    expect(window.location.hash).toBe("#/settings");
    expect(window.history.length).toBe(length);
    expect(window.history.state).toEqual({ seed: true });
  });
});
