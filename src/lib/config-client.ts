import type { ConfigSnapshot, UiConfig } from "../types/generated/bindings";
import { commands, events } from "../types/generated/bindings";

export interface UiConfigClient {
  fetchSnapshot(): Promise<ConfigSnapshot>;
  updateUi(ui: UiConfig): Promise<ConfigSnapshot>;
  listenChanged(
    listener: (snapshot: ConfigSnapshot) => void,
  ): Promise<() => void>;
}

export const uiConfigClient: UiConfigClient = {
  fetchSnapshot: async () => unwrap(await commands.configSnapshot()),
  updateUi: async (ui) => unwrap(await commands.configUpdateUi(ui)),
  listenChanged: async (listener) =>
    events.configChanged.listen((event) => listener(event.payload.snapshot)),
};

type ConfigCommandResult = Awaited<ReturnType<typeof commands.configSnapshot>>;

function unwrap(result: ConfigCommandResult): ConfigSnapshot {
  if (result.status === "error") {
    throw new Error(result.error.message);
  }
  return result.data;
}
