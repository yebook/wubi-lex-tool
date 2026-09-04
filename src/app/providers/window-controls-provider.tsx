import { createContext, useContext } from "react";
import type { ReactNode } from "react";

import { useWindowControls } from "../../hooks/use-window-controls";
import type { WindowClient } from "../../lib/window-client";

type WindowControlsValue = ReturnType<typeof useWindowControls>;

const WindowControlsContext = createContext<WindowControlsValue | null>(null);

export function WindowControlsProvider({
  children,
  client,
}: {
  children: ReactNode;
  client?: WindowClient;
}) {
  const controls = useWindowControls(client);
  return (
    <WindowControlsContext.Provider value={controls}>
      {children}
    </WindowControlsContext.Provider>
  );
}

export function useAppWindowControls(): WindowControlsValue {
  const value = useContext(WindowControlsContext);
  if (!value) {
    throw new Error(
      "useAppWindowControls must be used within WindowControlsProvider",
    );
  }
  return value;
}
