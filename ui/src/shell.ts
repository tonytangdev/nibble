import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";

import { previewView } from "./preview";
import type { View } from "./view";

/** What the popover can ask of the shell: the commands in crates/nibble-app/src/shell.rs. */
export type Command =
  | { name: "dismiss" }
  | { name: "celebrated" }
  | { name: "answer_autostart"; enable: boolean }
  | { name: "set_notifications"; on: boolean }
  | { name: "quit" };

/**
 * True inside the Tauri webview. A plain browser tab (`pnpm ui:dev`) has no shell, so it
 * shows the sample Views from preview.ts instead.
 */
const inShell = "__TAURI_INTERNALS__" in window;

export function getView(): Promise<View> {
  return inShell ? invoke<View>("get_view") : Promise.resolve(previewView());
}

/** Calls `handler` with every View the shell pushes. Returns how to stop listening. */
export function onView(handler: (view: View) => void): () => void {
  if (!inShell) return () => undefined;
  const unlisten = listen<View>("view", (event) => {
    handler(event.payload);
  });
  return () => {
    void unlisten.then((stop) => {
      stop();
    });
  };
}

export function send(command: Command): void {
  if (!inShell) return;
  const { name, ...args } = command;
  void invoke(name, args);
}

/** Opens `url` in the default browser, since the popover's webview cannot navigate. */
export function openExternal(url: string): void {
  if (inShell) void openUrl(url);
  else window.open(url, "_blank", "noopener");
}
