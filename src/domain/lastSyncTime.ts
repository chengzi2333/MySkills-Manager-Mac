export function formatLastSyncTime(
  value: string | undefined,
  locale: string,
  neverLabel: string,
  timeZone?: string,
): string {
  if (!value) {
    return neverLabel;
  }

  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }

  const formatter = new Intl.DateTimeFormat(locale, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
    ...(timeZone ? { timeZone } : {}),
  });

  return formatter.format(parsed);
}
