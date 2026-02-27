export function tagsFromInput(value: string): string[] {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

export function tagsToInput(tags: string[]): string {
  return tags.join(", ");
}

