// Times as WhatsApp Web writes them.
const sameDay = (a: Date, b: Date) =>
  a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();

export function clock(seconds: number): string {
  return new Date(seconds * 1000).toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
}

/** The chat list's time: the clock today, "Yesterday", the weekday this week, else the date. */
export function listTime(seconds: number, now = new Date()): string {
  if (!seconds) return "";
  const date = new Date(seconds * 1000);
  if (sameDay(date, now)) return clock(seconds);
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  if (sameDay(date, yesterday)) return "Yesterday";
  const week = new Date(now);
  week.setDate(now.getDate() - 6);
  if (date > week) return date.toLocaleDateString(undefined, { weekday: "long" });
  return date.toLocaleDateString(undefined, { day: "2-digit", month: "2-digit", year: "numeric" });
}

/** A date separator in the conversation: "Today", "Yesterday", the weekday, or the date. */
export function dayLabel(seconds: number, now = new Date()): string {
  const date = new Date(seconds * 1000);
  if (sameDay(date, now)) return "Today";
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  if (sameDay(date, yesterday)) return "Yesterday";
  const week = new Date(now);
  week.setDate(now.getDate() - 6);
  if (date > week) return date.toLocaleDateString(undefined, { weekday: "long" });
  return date.toLocaleDateString(undefined, { day: "2-digit", month: "2-digit", year: "numeric" });
}

export function dayKey(seconds: number): string {
  const d = new Date(seconds * 1000);
  return `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;
}
