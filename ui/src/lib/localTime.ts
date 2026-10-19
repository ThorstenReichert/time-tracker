// Local wall-clock times as the backend sends and takes them: "YYYY-MM-DDTHH:MM:SS", with no
// time zone. They stay text here; the backend converts them with the OS time zone, so a time in
// a DST gap or the repeated hour reaches the core and is rejected there (OQ-6).
import { addDays } from "./day";

/** The date and the time of day of a local time, the time as "HH:MM" (or "HH:MM:SS"). */
export function splitLocal(local: string): { date: string; time: string } {
  const [date = "", time = ""] = local.split("T");
  return { date, time: time.endsWith(":00") ? time.slice(0, 5) : time.slice(0, 8) };
}

/**
 * A typed time of day as "HH:MM:SS", or `null` if it isn't one. Takes "9", "9:05", "09.05",
 * "905", "0905" and "09:05:30".
 */
export function parseTime(text: string): string | null {
  const match = /^(\d{1,2})(?:[:.]?(\d{2}))?(?::(\d{2}))?$/.exec(text.trim());
  if (match === null) return null;
  const [h, m, s] = [match[1], match[2], match[3]].map((part) => Number(part ?? 0)) as [number, number, number];
  if (h > 23 || m > 59 || s > 59) return null;
  return [h, m, s].map((n) => String(n).padStart(2, "0")).join(":");
}

/** The local time for a date and a typed time of day, or `null` if the time can't be read. */
export function joinLocal(date: string, timeText: string): string | null {
  const time = parseTime(timeText);
  return time === null ? null : `${date}T${time}`;
}

/** "HH:MM" of a local time. */
export function formatClock(local: string): string {
  return local.slice(11, 16);
}

/** An entry's times for a list, e.g. "09:00–10:22", "22:00–01:30 (next day)" or "09:00–now". */
export function formatSpan(startLocal: string, endLocal: string | null): string {
  if (endLocal === null) return `${formatClock(startLocal)}–now`;
  const startDate = splitLocal(startLocal).date;
  const endDate = splitLocal(endLocal).date;
  const suffix = endDate === startDate ? "" : endDate === addDays(startDate, 1) ? " (next day)" : ` (${endDate})`;
  return `${formatClock(startLocal)}–${formatClock(endLocal)}${suffix}`;
}
