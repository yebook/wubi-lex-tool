import type {
  LaunchRequestedEvent,
  RuntimeSnapshot,
} from "../types/generated/bindings";
import { commands, events } from "../types/generated/bindings";

export interface RuntimeClient {
  fetchSnapshot(): Promise<RuntimeSnapshot>;
  listenLaunch(
    listener: (launch: LaunchRequestedEvent) => void,
  ): Promise<() => void>;
}

export const runtimeClient: RuntimeClient = {
  fetchSnapshot: () => commands.appRuntimeSnapshot(),
  listenLaunch: async (listener) =>
    events.appLaunchRequested.listen((event) => listener(event.payload)),
};
