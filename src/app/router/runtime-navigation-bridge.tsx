import { useLayoutEffect, useRef } from "react";

import { useAppRuntime } from "../providers/app-runtime-provider";
import { useAppNavigation } from "./navigation-provider";

export function RuntimeNavigationBridge({
  consumedLaunchSequence,
}: {
  consumedLaunchSequence: number;
}) {
  const runtime = useAppRuntime();
  const { navigateProductPath } = useAppNavigation();
  const consumed = useRef(consumedLaunchSequence);

  useLayoutEffect(() => {
    const launch = runtime.latestLaunch;
    if (!launch || launch.sequence <= consumed.current) {
      return;
    }
    consumed.current = launch.sequence;
    const path = launch.event.request.navigationPath;
    if (path) {
      navigateProductPath(path);
    }
  }, [navigateProductPath, runtime.latestLaunch]);

  return null;
}
