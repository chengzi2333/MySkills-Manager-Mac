export type SkillDocument = {
  frontmatter: Record<string, unknown>;
  body: string;
};

export type EditableSkillFrontmatter = {
  name: string;
  description: string;
  category: string;
  tags: string[];
  my_notes: string;
  last_updated: string;
  extra: Record<string, unknown>;
};

export type EditableSkillDocument = {
  frontmatter: EditableSkillFrontmatter;
  body: string;
};

const KNOWN_KEYS = new Set([
  "name",
  "description",
  "category",
  "tags",
  "my_notes",
  "last_updated",
]);

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function asTags(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((item) => String(item).trim())
    .filter((item) => item.length > 0);
}

function quoteYaml(value: string): string {
  return JSON.stringify(value);
}

function serializeYamlPrimitive(value: unknown): string {
  if (typeof value === "string") {
    return quoteYaml(value);
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (value === null || value === undefined) {
    return "null";
  }
  return quoteYaml(JSON.stringify(value));
}

export function toEditableDocument(raw: SkillDocument): EditableSkillDocument {
  const frontmatter = raw.frontmatter ?? {};
  const extra: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(frontmatter)) {
    if (!KNOWN_KEYS.has(key)) {
      extra[key] = value;
    }
  }

  return {
    frontmatter: {
      name: asString(frontmatter.name),
      description: asString(frontmatter.description),
      category: asString(frontmatter.category),
      tags: asTags(frontmatter.tags),
      my_notes: asString(frontmatter.my_notes),
      last_updated: asString(frontmatter.last_updated),
      extra,
    },
    body: raw.body ?? "",
  };
}

export function fromEditableDocument(doc: EditableSkillDocument): string {
  const lines: string[] = [];
  const meta = doc.frontmatter;

  if (meta.name) lines.push(`name: ${quoteYaml(meta.name)}`);
  if (meta.description) lines.push(`description: ${quoteYaml(meta.description)}`);
  if (meta.category) lines.push(`category: ${quoteYaml(meta.category)}`);

  if (meta.tags.length > 0) {
    lines.push("tags:");
    for (const tag of meta.tags) {
      lines.push(`  - ${quoteYaml(tag)}`);
    }
  }

  if (meta.my_notes) lines.push(`my_notes: ${quoteYaml(meta.my_notes)}`);
  if (meta.last_updated) lines.push(`last_updated: ${quoteYaml(meta.last_updated)}`);

  const extraKeys = Object.keys(meta.extra).sort();
  for (const key of extraKeys) {
    lines.push(`${key}: ${serializeYamlPrimitive(meta.extra[key])}`);
  }

  const normalizedBody = doc.body.replace(/\r\n/g, "\n").trimStart();
  return `---\n${lines.join("\n")}\n---\n\n${normalizedBody}`;
}
