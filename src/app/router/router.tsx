import type { ComponentType } from "react";
import type { RouteObject } from "react-router";
import { createHashRouter, createMemoryRouter, Navigate } from "react-router";

import { AppShell } from "../layout/AppShell";
import type { InitialNavigation } from "../providers/app-runtime-provider";
import { LearningRoute } from "../../routes/learning/LearningRoute";
import { LexiconsRoute } from "../../routes/lexicons/LexiconsRoute";
import { LookupRoute } from "../../routes/lookup/LookupRoute";
import { OverviewRoute } from "../../routes/overview/OverviewRoute";
import { PhrasesRoute } from "../../routes/phrases/PhrasesRoute";
import { RadicalsRoute } from "../../routes/radicals/RadicalsRoute";
import { SettingsRoute } from "../../routes/settings/SettingsRoute";
import { overviewPath, routeCatalog } from "./catalog";
import type { RouteId } from "./catalog";
import { NavigationProvider } from "./navigation-provider";
import { RuntimeNavigationBridge } from "./runtime-navigation-bridge";
import { UnknownRoute } from "./unknown-route";

const routeComponents = {
  overview: OverviewRoute,
  lexicons: LexiconsRoute,
  phrases: PhrasesRoute,
  lookup: LookupRoute,
  radicals: RadicalsRoute,
  learning: LearningRoute,
  settings: SettingsRoute,
} satisfies Record<RouteId, ComponentType>;

export function createRouteObjects(initial: InitialNavigation): RouteObject[] {
  return [
    {
      element: (
        <NavigationProvider initialWarning={initial.warning}>
          <RuntimeNavigationBridge
            consumedLaunchSequence={initial.consumedLaunchSequence}
          />
          <AppShell />
        </NavigationProvider>
      ),
      children: [
        { path: "/", element: <Navigate replace to={overviewPath} /> },
        ...routeCatalog.map((route) => {
          const RouteComponent = routeComponents[route.id];
          return { path: route.path, element: <RouteComponent /> };
        }),
        { path: "*", element: <UnknownRoute /> },
      ],
    },
  ];
}

export function createHashAppRouter(initial: InitialNavigation) {
  replaceInitialHash(initial.path);
  return createHashRouter(createRouteObjects(initial));
}

export function createMemoryAppRouter(
  initial: InitialNavigation,
  initialEntries: string[] = [initial.path],
) {
  return createMemoryRouter(createRouteObjects(initial), { initialEntries });
}

export function replaceInitialHash(path: string): void {
  const url = new URL(window.location.href);
  url.hash = path;
  window.history.replaceState(window.history.state, "", url);
}
