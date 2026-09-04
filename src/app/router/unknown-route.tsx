import { useLayoutEffect } from "react";
import { useLocation } from "react-router";

import { useAppNavigation } from "./navigation-provider";

export function UnknownRoute() {
  const location = useLocation();
  const { navigateProductPath } = useAppNavigation();

  useLayoutEffect(() => {
    navigateProductPath(location.pathname);
  }, [location.pathname, navigateProductPath]);

  return null;
}
