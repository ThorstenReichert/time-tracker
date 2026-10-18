import type { RuleDto, Separator } from "../bindings";

/** Elapsed seconds as "h:mm:ss" (the running timer, R5). Negative values show as 0:00:00. */
export function formatElapsed(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

/** The character between whole and fractional hours. */
export type DecimalMark = "." | ",";

/**
 * The decimal mark for the separator setting (R15). `auto` follows the OS locale, which the
 * WebView reports as its default locale (AC-3).
 */
export function decimalMark(setting: Separator, locale?: string): DecimalMark {
  if (setting === "dot") return ".";
  if (setting === "comma") return ",";
  const part = new Intl.NumberFormat(locale).formatToParts(1.5).find((p) => p.type === "decimal");
  return part?.value === "," ? "," : ".";
}

/**
 * Seconds as decimal hours with 2 decimals (R15), e.g. 4920 → "1.37". Integer math, so there
 * are no float surprises: hundredths = (seconds·100 + 1800) div 3600, half away from zero for
 * negative values. This is display only; every calculation stays in seconds (spec §2).
 */
export function formatHours(seconds: number, mark: DecimalMark): string {
  const hundredths = Math.floor((Math.abs(seconds) * 100 + 1800) / 3600);
  const sign = seconds < 0 && hundredths > 0 ? "-" : "";
  const whole = Math.floor(hundredths / 100);
  const fraction = String(hundredths % 100).padStart(2, "0");
  return `${sign}${whole}${mark}${fraction}`;
}

/** A difference in hours with its sign, e.g. "+0.03" or "-0.12"; zero has none. */
export function formatDiff(seconds: number, mark: DecimalMark): string {
  const text = formatHours(seconds, mark);
  return seconds > 0 && text !== formatHours(0, mark) ? `+${text}` : text;
}

/** A client's rounding rule, e.g. "15 min · nearest · per project" (R17). */
export function formatRule(rule: RuleDto): string {
  return `${rule.unit} min · ${rule.direction} · per ${rule.level}`;
}
