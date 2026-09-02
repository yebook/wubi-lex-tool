import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import type { WindowClient } from "../lib/window-client";
import { windowClient } from "../lib/window-client";
import type {
  RuntimeNotice,
  WindowControlIntent,
  WindowStateSnapshot,
} from "../types/generated/bindings";

const MAX_LIVE_NOTICES = 8;

export function mergeWindowState(
  current: WindowStateSnapshot | null,
  incoming: WindowStateSnapshot,
): WindowStateSnapshot {
  return current && current.revision > incoming.revision ? current : incoming;
}

export function useWindowControls(client: WindowClient = windowClient) {
  const { t } = useTranslation("window");
  const [snapshot, setSnapshot] = useState<WindowStateSnapshot | null>(null);
  const [notices, setNotices] = useState<RuntimeNotice[]>([]);
  const [listenerWarning, setListenerWarning] = useState<string | null>(null);
  const [commandWarning, setCommandWarning] = useState<string | null>(null);

  useEffect(() => {
    let disposed = false;
    const unlisten: Array<() => void> = [];

    const initialize = async () => {
      try {
        const stopState = await client.listenState((incoming) => {
          if (!disposed) {
            setSnapshot((current) => mergeWindowState(current, incoming));
          }
        });
        if (disposed) {
          stopState();
          return;
        }
        unlisten.push(stopState);
      } catch (error) {
        if (!disposed) {
          setListenerWarning(
            windowWarningMessage(error, t("warning.stateListener")),
          );
        }
      }
      try {
        const stopNotice = await client.listenNotice((notice) => {
          if (!disposed) {
            setNotices((current) => appendNotice(current, notice));
          }
        });
        if (disposed) {
          stopNotice();
          return;
        }
        unlisten.push(stopNotice);
      } catch (error) {
        if (!disposed) {
          setListenerWarning(
            windowWarningMessage(error, t("warning.noticeListener")),
          );
        }
      }
      try {
        const initial = await client.fetchState();
        if (!disposed) {
          setSnapshot((current) => mergeWindowState(current, initial));
        }
      } catch (error) {
        if (!disposed) {
          setListenerWarning(
            windowWarningMessage(error, t("warning.stateSnapshot")),
          );
        }
      }
    };

    void initialize();
    return () => {
      disposed = true;
      for (const stopListening of unlisten) {
        stopListening();
      }
    };
  }, [client, t]);

  const control = useCallback(
    async (intent: WindowControlIntent) => {
      try {
        const next = await client.control(intent);
        setSnapshot((current) => mergeWindowState(current, next));
        setCommandWarning(null);
      } catch (error) {
        setCommandWarning(windowWarningMessage(error, t("warning.command")));
      }
    },
    [client, t],
  );

  return {
    snapshot,
    notices,
    warning: commandWarning ?? listenerWarning,
    control,
  };
}

function appendNotice(
  current: RuntimeNotice[],
  incoming: RuntimeNotice,
): RuntimeNotice[] {
  if (
    current.some(
      (notice) =>
        notice.code === incoming.code && notice.detail === incoming.detail,
    )
  ) {
    return current;
  }
  return [...current, incoming].slice(-MAX_LIVE_NOTICES);
}

function windowWarningMessage(error: unknown, fallback: string): string {
  return error instanceof Error && error.message ? error.message : fallback;
}
