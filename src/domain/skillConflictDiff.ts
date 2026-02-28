export type SkillDiffLineKind = "context" | "removed" | "added";

export type SkillDiffLine = {
  kind: SkillDiffLineKind;
  text: string;
};

export type SkillDiffResult = {
  hasChanges: boolean;
  added: number;
  removed: number;
  lines: SkillDiffLine[];
  truncated: boolean;
  hiddenLineCount: number;
};

function normalizeLines(content: string): string[] {
  const normalized = content.replace(/\r\n/g, "\n");
  const lines = normalized.split("\n");
  if (lines.length > 0 && lines[lines.length - 1] === "") {
    lines.pop();
  }
  return lines;
}

function lcsTable(baseLines: string[], incomingLines: string[]): number[][] {
  const n = baseLines.length;
  const m = incomingLines.length;
  const table = Array.from({ length: n + 1 }, () => Array<number>(m + 1).fill(0));

  for (let i = n - 1; i >= 0; i -= 1) {
    for (let j = m - 1; j >= 0; j -= 1) {
      if (baseLines[i] === incomingLines[j]) {
        table[i][j] = table[i + 1][j + 1] + 1;
      } else {
        table[i][j] = Math.max(table[i + 1][j], table[i][j + 1]);
      }
    }
  }
  return table;
}

export function buildSkillDiff(
  baseContent: string,
  incomingContent: string,
  maxLines = 280,
): SkillDiffResult {
  const baseLines = normalizeLines(baseContent);
  const incomingLines = normalizeLines(incomingContent);
  const table = lcsTable(baseLines, incomingLines);

  const allLines: SkillDiffLine[] = [];
  let added = 0;
  let removed = 0;
  let i = 0;
  let j = 0;

  while (i < baseLines.length && j < incomingLines.length) {
    if (baseLines[i] === incomingLines[j]) {
      allLines.push({ kind: "context", text: baseLines[i] });
      i += 1;
      j += 1;
      continue;
    }
    if (table[i + 1][j] >= table[i][j + 1]) {
      allLines.push({ kind: "removed", text: baseLines[i] });
      removed += 1;
      i += 1;
      continue;
    }
    allLines.push({ kind: "added", text: incomingLines[j] });
    added += 1;
    j += 1;
  }

  while (i < baseLines.length) {
    allLines.push({ kind: "removed", text: baseLines[i] });
    removed += 1;
    i += 1;
  }
  while (j < incomingLines.length) {
    allLines.push({ kind: "added", text: incomingLines[j] });
    added += 1;
    j += 1;
  }

  if (maxLines <= 0 || allLines.length <= maxLines) {
    return {
      hasChanges: added + removed > 0,
      added,
      removed,
      lines: allLines,
      truncated: false,
      hiddenLineCount: 0,
    };
  }

  return {
    hasChanges: added + removed > 0,
    added,
    removed,
    lines: allLines.slice(0, maxLines),
    truncated: true,
    hiddenLineCount: allLines.length - maxLines,
  };
}
