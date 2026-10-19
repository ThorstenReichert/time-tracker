import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { UiError } from "../bindings";

/** A command as the page sent it. */
export interface Call {
  cmd: string;
  args: Record<string, unknown>;
}

/** Answers one command; `any`, because each handler reads its own command's arguments. */
export type Handler = (args: any) => unknown;

/**
 * Fakes the Tauri backend for a component test: answers each command (by its Rust name, e.g.
 * `quick_switch`) with its handler and records every call. Events are faked too, so `emit`
 * from `@tauri-apps/api/event` reaches the page's `listen` handlers. A handler that throws a
 * `uiError(...)` makes the command fail the way the backend's `UiError` does.
 */
export function fakeBackend(handlers: Record<string, Handler>, window = "main") {
  const calls: Call[] = [];
  mockWindows(window);
  mockIPC(
    async (cmd, payload) => {
      const args = (payload ?? {}) as Record<string, unknown>;
      calls.push({ cmd, args });
      const handler = handlers[cmd];
      if (handler === undefined) throw new Error(`fake backend: no handler for ${cmd}`);
      return handler(args);
    },
    { shouldMockEvents: true },
  );
  return {
    calls,
    handlers,
    /** The arguments of each call of `cmd`, in order. */
    called: (cmd: string) => calls.filter((c) => c.cmd === cmd).map((c) => c.args),
  };
}

/** A rejection as the backend sends it (not an `Error`, so the bindings return it as data). */
export function uiError(message: string, kind: UiError["kind"] = "domain"): UiError {
  return { kind, message };
}

/** A promise the test resolves when it wants, e.g. to hold a search back. */
export function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => (resolve = r));
  return { promise, resolve };
}
