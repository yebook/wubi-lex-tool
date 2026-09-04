import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";

import type { RuntimeClient } from "../../lib/runtime-client";
import { runtimeClient } from "../../lib/runtime-client";
import { boundVisibleText } from "../../lib/visible-text";
import { mergeLatestLaunch, runtimeErrorMessage } from "../../runtime-view";
import type {
  LaunchRequestedEvent,
  RuntimeSnapshot,
} from "../../types/generated/bindings";
import type { CanonicalRoutePath } from "../router/catalog";
import { overviewPath } from "../router/catalog";
import { readCurrentHashPath, validateProductPath } from "../router/path";

export type RuntimeLoadState =
  | { status: "loading" }
  | { status: "ready"; snapshot: RuntimeSnapshot }
  | { status: "error"; message: string };

export interface SequencedLaunch {
  sequence: number;
  event: LaunchRequestedEvent;
}

export interface InitialNavigation {
  path: CanonicalRoutePath;
  warning: string | null;
  consumedLaunchSequence: number;
}

export interface AppRuntimeContextValue {
  loadState: RuntimeLoadState;
  latestLaunch: SequencedLaunch | null;
  latestNavigationLaunch: SequencedLaunch | null;
  listenerWarning: string | null;
  refreshWarning: string | null;
  refresh(showLoading?: boolean): Promise<void>;
}

interface AppRuntimeProviderProps {
  children: ReactNode;
  client?: RuntimeClient;
}

const AppRuntimeContext = createContext<AppRuntimeContextValue | null>(null);

export function AppRuntimeProvider({
  children,
  client = runtimeClient,
}: AppRuntimeProviderProps) {
  const { t } = useTranslation("runtime");
  const [loadState, setLoadState] = useState<RuntimeLoadState>({
    status: "loading",
  });
  const [latestLaunch, setLatestLaunch] = useState<SequencedLaunch | null>(
    null,
  );
  const [latestNavigationLaunch, setLatestNavigationLaunch] =
    useState<SequencedLaunch | null>(null);
  const [listenerWarning, setListenerWarning] = useState<string | null>(null);
  const [refreshWarning, setRefreshWarning] = useState<string | null>(null);
  const latestEvent = useRef<SequencedLaunch | null>(null);
  const launchSequence = useRef(0);
  const hasReadySnapshot = useRef(false);
  const lastSnapshot = useRef<RuntimeSnapshot | null>(null);
  const refreshGeneration = useRef(0);
  const mounted = useRef(true);

  const refresh = useCallback(
    async (showLoading = true) => {
      const generation = ++refreshGeneration.current;
      const sequenceAtStart = launchSequence.current;
      if (showLoading && mounted.current) {
        setLoadState({ status: "loading" });
      }
      try {
        const snapshot = await client.fetchSnapshot();
        if (!mounted.current || generation !== refreshGeneration.current) {
          return;
        }
        const launchDuringRequest =
          launchSequence.current === sequenceAtStart
            ? null
            : (latestEvent.current?.event ?? null);
        const mergedSnapshot = mergeLatestLaunch(snapshot, launchDuringRequest);
        lastSnapshot.current = mergedSnapshot;
        setLoadState({
          status: "ready",
          snapshot: mergedSnapshot,
        });
        hasReadySnapshot.current = true;
        setRefreshWarning(null);
      } catch (error) {
        if (!mounted.current || generation !== refreshGeneration.current) {
          return;
        }
        if (lastSnapshot.current) {
          setLoadState({ status: "ready", snapshot: lastSnapshot.current });
          setRefreshWarning(t("warning.refresh"));
        } else if (showLoading) {
          setLoadState({
            status: "error",
            message: boundVisibleText(runtimeErrorMessage(error, t)),
          });
        } else {
          setRefreshWarning(t("warning.refresh"));
        }
      }
    },
    [client, t],
  );

  useEffect(() => {
    mounted.current = true;
    let disposed = false;
    let stopListening: (() => void) | undefined;

    const initialize = async () => {
      try {
        const stop = await client.listenLaunch((event) => {
          if (disposed) {
            return;
          }
          const sequenced = {
            sequence: ++launchSequence.current,
            event,
          } satisfies SequencedLaunch;
          latestEvent.current = sequenced;
          setLatestLaunch(sequenced);
          if (event.request.navigationPath) {
            setLatestNavigationLaunch(sequenced);
          }
          setLoadState((current) => {
            if (current.status !== "ready") {
              return current;
            }
            const mergedSnapshot = mergeLatestLaunch(current.snapshot, event);
            lastSnapshot.current = mergedSnapshot;
            return { status: "ready", snapshot: mergedSnapshot };
          });
          if (hasReadySnapshot.current) {
            void refresh(false);
          }
        });
        if (disposed) {
          stop();
          return;
        }
        stopListening = stop;
      } catch {
        if (!disposed) {
          setListenerWarning(t("warning.listener"));
        }
      }

      if (!disposed) {
        await refresh();
      }
    };

    void initialize();
    return () => {
      disposed = true;
      mounted.current = false;
      refreshGeneration.current += 1;
      stopListening?.();
    };
  }, [client, refresh, t]);

  const value = useMemo<AppRuntimeContextValue>(
    () => ({
      loadState,
      latestLaunch,
      latestNavigationLaunch,
      listenerWarning,
      refreshWarning,
      refresh,
    }),
    [
      latestLaunch,
      latestNavigationLaunch,
      listenerWarning,
      loadState,
      refresh,
      refreshWarning,
    ],
  );

  return (
    <AppRuntimeContext.Provider value={value}>
      {children}
    </AppRuntimeContext.Provider>
  );
}

export function useAppRuntime(): AppRuntimeContextValue {
  const value = useContext(AppRuntimeContext);
  if (!value) {
    throw new Error("useAppRuntime must be used within AppRuntimeProvider");
  }
  return value;
}

export function resolveInitialNavigation(
  runtime: AppRuntimeContextValue,
  hashPath: string | null,
): InitialNavigation {
  const snapshot =
    runtime.loadState.status === "ready" ? runtime.loadState.snapshot : null;
  const latestEventPath =
    runtime.latestNavigationLaunch?.event.request.navigationPath;
  const candidate =
    latestEventPath ??
    snapshot?.latestSecondaryLaunch?.request.navigationPath ??
    snapshot?.primaryLaunch.request.navigationPath ??
    hashPath ??
    overviewPath;
  const result = validateProductPath(candidate);

  return {
    path: result.path,
    warning: result.warning,
    consumedLaunchSequence: runtime.latestLaunch?.sequence ?? 0,
  };
}

export function resolveBrowserInitialNavigation(
  runtime: AppRuntimeContextValue,
): InitialNavigation {
  return resolveInitialNavigation(runtime, readCurrentHashPath());
}
