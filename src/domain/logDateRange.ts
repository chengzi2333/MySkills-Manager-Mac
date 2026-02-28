export function toIsoStart(value: string): string | undefined {
  return value ? `${value}T00:00:00Z` : undefined;
}

export function toIsoEnd(value: string): string | undefined {
  return value ? `${value}T23:59:59Z` : undefined;
}
