import type {
  RuntimeNotice,
  WindowControlIntent,
  WindowStateSnapshot,
} from "../types/generated/bindings";
import { commands, events } from "../types/generated/bindings";

export interface WindowClient {
  fetchState(): Promise<WindowStateSnapshot>;
  control(intent: WindowControlIntent): Promise<WindowStateSnapshot>;
  listenState(listener: (snapshot: WindowStateSnapshot) => void): Promise<() => void>;
  listenNotice(listener: (notice: RuntimeNotice) => void): Promise<() => void>;
}

export const windowClient: WindowClient = {
  fetchState: () => commands.windowState(),
  control: async (intent) => {
    const result = await commands.windowControl(intent);
    if (result.status === "error") {
      throw new Error(result.error.message);
    }
    return result.data;
  },
  listenState: async (listener) =>
    events.windowStateChanged.listen((event) => listener(event.payload.snapshot)),
  listenNotice: async (listener) =>
    events.appRuntimeNotice.listen((event) => listener(event.payload.notice)),
};
