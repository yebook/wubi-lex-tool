import { i18n } from "../../i18n";
import { boundVisibleText } from "../../lib/visible-text";
import { overviewPath, routeForPath } from "./catalog";
import type { CanonicalRoutePath } from "./catalog";

const MAX_VISIBLE_PATH_SCALARS = 192;

export type ProductPathResult =
  | { kind: "canonical"; path: CanonicalRoutePath; warning: null }
  | {
      kind: "redirect";
      path: typeof overviewPath;
      warning: string | null;
    };

export function validateProductPath(path: string): ProductPathResult {
  if (path === "/") {
    return { kind: "redirect", path: overviewPath, warning: null };
  }

  const route = routeForPath(path);
  if (route) {
    return { kind: "canonical", path: route.path, warning: null };
  }

  const visiblePath = boundVisibleText(
    path || i18n.t("shell:navigation.emptyPath"),
    MAX_VISIBLE_PATH_SCALARS,
  );
  return {
    kind: "redirect",
    path: overviewPath,
    warning: boundVisibleText(
      i18n.t("shell:navigation.unknownPath", { path: visiblePath }),
    ),
  };
}

export function readCurrentHashPath(location: Location = window.location) {
  const value = location.hash.startsWith("#")
    ? location.hash.slice(1)
    : location.hash;
  return value || null;
}
