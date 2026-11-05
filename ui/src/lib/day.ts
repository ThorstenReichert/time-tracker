// Calendar dates as the backend sends them: "YYYY-MM-DD", a local date with no time zone.
// The arithmetic runs in UTC, so a DST change can never shift the date.

/** Year, month (1–12) and day of "YYYY-MM-DD". */
function parts(day: string): [number, number, number] {
  const [y = NaN, m = NaN, d = NaN] = day.split("-").map(Number);
  return [y, m, d];
}

function toUtc(day: string): Date {
  const [y, m, d] = parts(day);
  return new Date(Date.UTC(y, m - 1, d));
}

function fromUtc(date: Date): string {
  const y = String(date.getUTCFullYear()).padStart(4, "0");
  const m = String(date.getUTCMonth() + 1).padStart(2, "0");
  const d = String(date.getUTCDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

/** The date `days` days after `day` (before, if negative). */
export function addDays(day: string, days: number): string {
  const date = toUtc(day);
  date.setUTCDate(date.getUTCDate() + days);
  return fromUtc(date);
}

/** The date for a DatePicker, which works with local `Date`s at midnight. */
export function toPickerDate(day: string): Date {
  const [y, m, d] = parts(day);
  return new Date(y, m - 1, d);
}

/** The date a DatePicker picked. */
export function fromPickerDate(date: Date): string {
  return fromUtc(new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate())));
}

/** The date for a heading, e.g. "Sunday, 18 October 2026" (in the OS locale). */
export function formatDay(day: string, locale?: string): string {
  return toUtc(day).toLocaleDateString(locale, {
    timeZone: "UTC",
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  });
}

/** A date in short form, e.g. "18 Oct 2026" (in the OS locale). */
export function formatShortDay(day: string, locale?: string): string {
  return toUtc(day).toLocaleDateString(locale, { timeZone: "UTC", day: "numeric", month: "short", year: "numeric" });
}
