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

import type { UiConfigClient } from "../../lib/config-client";
import { uiConfigClient } from "../../lib/config-client";
import {
  applyUiAppearance,
  browserAppearanceEnvironment,
  nativeThemeForPreference,
  normalizeUiConfig,
  readBootstrapUi,
  synchronizeUiAppearance,
} from "../../lib/ui-appearance";
import type {
  AppearanceEnvironment,
  ResolvedUiConfig,
} from "../../lib/ui-appearance";
import type {
  AppLocale,
  ConfigSnapshot,
  Density,
  ThemePreference,
} from "../../types/generated/bindings";

type UiPreferencesStatus = "loading" | "ready" | "failed";
type UiPreferencePatch = Partial<
  Pick<ResolvedUiConfig, "theme" | "density" | "locale">
>;

interface UpdateJob {
  patch: UiPreferencePatch;
}

export interface UiPreferencesContextValue {
  status: UiPreferencesStatus;
  ui: ResolvedUiConfig;
  warning: string | null;
  setTheme(theme: ThemePreference): Promise<void>;
  setDensity(density: Density): Promise<void>;
  setLocale(locale: AppLocale): Promise<void>;
  clearWarning(): void;
}

interface UiPreferencesProviderProps {
  children: ReactNode;
  client?: UiConfigClient;
  appearanceEnvironment?: AppearanceEnvironment;
}

interface ViewState {
  status: UiPreferencesStatus;
  ui: ResolvedUiConfig;
  warning: string | null;
}

interface ConfirmedUi {
  revision: number;
  ui: ResolvedUiConfig;
}

const UiPreferencesContext = createContext<UiPreferencesContextValue | null>(
  null,
);

export function UiPreferencesProvider({
  children,
  client = uiConfigClient,
  appearanceEnvironment,
}: UiPreferencesProviderProps) {
  const { i18n: translation } = useTranslation();
  const environment = useMemo(
    () => appearanceEnvironment ?? browserAppearanceEnvironment(),
    [appearanceEnvironment],
  );
  const bootstrapUi = useMemo(
    () => readBootstrapUi(environment.root),
    [environment],
  );
  const [view, setView] = useState<ViewState>({
    status: "loading",
    ui: bootstrapUi,
    warning: null,
  });
  const confirmed = useRef<ConfirmedUi>({ revision: -1, ui: bootstrapUi });
  const pending = useRef<UpdateJob[]>([]);
  const updateQueue = useRef<Promise<void>>(Promise.resolve());
  const mounted = useRef(true);

  const project = useCallback(
    (ui: ResolvedUiConfig) => {
      applyUiAppearance(ui, environment);
      if (
        (translation.resolvedLanguage ?? translation.language) !== ui.locale
      ) {
        void translation.changeLanguage(ui.locale);
      }
    },
    [environment, translation],
  );

  const refreshOptimisticView = useCallback(
    (warning?: string | null) => {
      const ui = applyPending(confirmed.current.ui, pending.current);
      project(ui);
      if (mounted.current) {
        setView((current) => ({
          status: confirmed.current.revision >= 0 ? "ready" : current.status,
          ui,
          warning: warning === undefined ? current.warning : warning,
        }));
      }
    },
    [project],
  );

  const mergeSnapshot = useCallback(
    (snapshot: ConfigSnapshot) => {
      if (snapshot.revision < confirmed.current.revision) {
        return;
      }
      confirmed.current = {
        revision: snapshot.revision,
        ui: normalizeUiConfig(snapshot.config.ui),
      };
      const notice = snapshot.notices.at(-1);
      refreshOptimisticView(
        notice ? formatConfigNotice(notice.message, notice.detail) : undefined,
      );
    },
    [refreshOptimisticView],
  );

  useEffect(() => {
    mounted.current = true;
    let disposed = false;
    let stopListening: (() => void) | undefined;

    const initialize = async () => {
      try {
        const stop = await client.listenChanged((snapshot) => {
          if (!disposed) {
            mergeSnapshot(snapshot);
          }
        });
        if (disposed) {
          stop();
          return;
        }
        stopListening = stop;
      } catch (error) {
        if (!disposed) {
          setView((current) => ({
            ...current,
            warning: visibleError(
              error,
              translation.t("ui:preferences.listenerFailed"),
            ),
          }));
        }
      }

      try {
        const snapshot = await client.fetchSnapshot();
        if (!disposed) {
          mergeSnapshot(snapshot);
        }
      } catch (error) {
        if (!disposed) {
          setView((current) => ({
            ...current,
            status: confirmed.current.revision >= 0 ? current.status : "failed",
            warning: visibleError(
              error,
              translation.t("ui:preferences.snapshotFailed"),
            ),
          }));
        }
      }
    };

    void initialize();
    return () => {
      disposed = true;
      mounted.current = false;
      stopListening?.();
    };
  }, [client, mergeSnapshot, translation]);

  useEffect(
    () => synchronizeUiAppearance(view.ui, environment),
    [environment, view.ui],
  );

  useEffect(() => {
    if (!environment.setNativeTheme) {
      return;
    }

    let disposed = false;
    void environment
      .setNativeTheme(nativeThemeForPreference(view.ui.theme))
      .catch((error: unknown) => {
        if (!disposed && mounted.current) {
          setView((current) => ({
            ...current,
            warning: visibleError(
              error,
              translation.t("ui:preferences.nativeThemeFailed"),
            ),
          }));
        }
      });
    return () => {
      disposed = true;
    };
  }, [environment, translation, view.ui.theme]);

  const enqueue = useCallback(
    (patch: UiPreferencePatch): Promise<void> => {
      const job = { patch } satisfies UpdateJob;
      pending.current.push(job);
      refreshOptimisticView();

      const persist = async () => {
        const requested = { ...confirmed.current.ui, ...job.patch };
        try {
          const snapshot = await client.updateUi(requested);
          mergeSnapshot(snapshot);
          pending.current = pending.current.filter(
            (current) => current !== job,
          );
          refreshOptimisticView();
        } catch (error) {
          pending.current = pending.current.filter(
            (current) => current !== job,
          );
          refreshOptimisticView(
            visibleError(error, translation.t("ui:preferences.updateFailed")),
          );
        }
      };

      const result = updateQueue.current.then(persist);
      updateQueue.current = result.catch(() => undefined);
      return result;
    },
    [client, mergeSnapshot, refreshOptimisticView, translation],
  );

  const value = useMemo<UiPreferencesContextValue>(
    () => ({
      ...view,
      setTheme: (theme) => enqueue({ theme }),
      setDensity: (density) => enqueue({ density }),
      setLocale: (locale) => enqueue({ locale }),
      clearWarning: () => setView((current) => ({ ...current, warning: null })),
    }),
    [enqueue, view],
  );

  return (
    <UiPreferencesContext.Provider value={value}>
      {children}
    </UiPreferencesContext.Provider>
  );
}

export function useUiPreferences(): UiPreferencesContextValue {
  const value = useContext(UiPreferencesContext);
  if (!value) {
    throw new Error(
      "useUiPreferences must be used within UiPreferencesProvider",
    );
  }
  return value;
}

function applyPending(
  base: ResolvedUiConfig,
  jobs: UpdateJob[],
): ResolvedUiConfig {
  return jobs.reduce((current, job) => ({ ...current, ...job.patch }), base);
}

function visibleError(error: unknown, fallback: string): string {
  const message =
    error instanceof Error && error.message ? error.message : fallback;
  return [...message].slice(0, 512).join("");
}

function formatConfigNotice(message: string, detail: string | null): string {
  return [...(detail ? `${message} ${detail}` : message)]
    .slice(0, 512)
    .join("");
}
