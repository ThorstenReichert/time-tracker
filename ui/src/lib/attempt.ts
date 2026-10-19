import { useToast } from "primevue/usetoast";
import { errorMessage } from "./api";

/**
 * For one-click actions (start, switch, stop): runs the action and shows a failure as a toast.
 * Resolves to whether it succeeded. Forms show their errors inline instead.
 */
export function useAttempt() {
  const toast = useToast();
  return async function attempt(action: () => Promise<unknown>): Promise<boolean> {
    try {
      await action();
      return true;
    } catch (e) {
      toast.add({ severity: "error", summary: "Nothing changed", detail: errorMessage(e), life: 8000 });
      return false;
    }
  };
}
