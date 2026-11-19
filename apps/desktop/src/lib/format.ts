// Display formatting. Timestamps are unix ms and only used for display.

export function formatNoteDate(ms: number, now: Date = new Date(), locale?: string): string {
  const date = new Date(ms);
  const sameDay = date.toDateString() === now.toDateString();
  if (sameDay) {
    return new Intl.DateTimeFormat(locale, { hour: "numeric", minute: "2-digit" }).format(date);
  }
  const sameYear = date.getFullYear() === now.getFullYear();
  return new Intl.DateTimeFormat(locale, {
    day: "numeric",
    month: "short",
    ...(sameYear ? {} : { year: "numeric" }),
  }).format(date);
}

export function displayTitle(title: string): string {
  return title.trim() === "" ? "Untitled" : title;
}
