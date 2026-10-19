import type { UiError } from "../bindings";

/** The result shape of every generated command. */
export type CommandResult<T> = { status: "ok"; data: T } | { status: "error"; error: UiError };

/** Thrown by `call` when a command is rejected or fails. */
export class CommandError extends Error {
  readonly kind: UiError["kind"];

  constructor(error: UiError) {
    super(error.message);
    this.kind = error.kind;
  }
}

/** Awaits a generated command and returns its data, or throws a `CommandError`. */
export async function call<T>(result: Promise<CommandResult<T>>): Promise<T> {
  const r = await result;
  if (r.status === "error") throw new CommandError(r.error);
  return r.data;
}

/** The message to show for any error. */
export function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
