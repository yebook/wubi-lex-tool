import type { LucideIcon } from "lucide-react";

import {
  BookOpenText,
  BookType,
  GraduationCap,
  LayoutDashboard,
  Search,
  Settings,
  Shapes,
} from "../../icons/ui";
import type { AppFeatureId } from "../../types/generated/bindings";

interface RouteCatalogEntry {
  id: string;
  path: `/${string}`;
  labelKey: `routes.${string}`;
  icon: LucideIcon;
  feature?: AppFeatureId;
}

export const routeCatalog = [
  {
    id: "overview",
    path: "/overview",
    labelKey: "routes.overview",
    icon: LayoutDashboard,
  },
  {
    id: "lexicons",
    path: "/lexicons",
    labelKey: "routes.lexicons",
    icon: BookOpenText,
    feature: "lexiconRead",
  },
  {
    id: "phrases",
    path: "/phrases",
    labelKey: "routes.phrases",
    icon: BookType,
    feature: "phraseRead",
  },
  {
    id: "lookup",
    path: "/lookup",
    labelKey: "routes.lookup",
    icon: Search,
    feature: "reverseLookup",
  },
  {
    id: "radicals",
    path: "/radicals",
    labelKey: "routes.radicals",
    icon: Shapes,
    feature: "radicalReference",
  },
  {
    id: "learning",
    path: "/learning",
    labelKey: "routes.learning",
    icon: GraduationCap,
    feature: "selfLearning",
  },
  {
    id: "settings",
    path: "/settings",
    labelKey: "routes.settings",
    icon: Settings,
  },
] as const satisfies readonly RouteCatalogEntry[];

export type RouteDefinition = (typeof routeCatalog)[number];
export type RouteId = RouteDefinition["id"];
export type CanonicalRoutePath = RouteDefinition["path"];

export const overviewPath: CanonicalRoutePath = routeCatalog[0].path;

const routesByPath = new Map<CanonicalRoutePath, RouteDefinition>(
  routeCatalog.map((route) => [route.path, route]),
);

export function routeForPath(path: string): RouteDefinition | undefined {
  return routesByPath.get(path as CanonicalRoutePath);
}
