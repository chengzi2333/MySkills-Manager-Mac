export type PathPickerTarget = "skills" | "rules";

export type ToolPathDraft = {
  skillsDir: string;
  rulesPath: string;
};

export type PathPickerOptions = {
  directory: boolean;
  multiple: boolean;
  defaultPath: string;
};

export function buildPathPickerOptions(
  target: PathPickerTarget,
  defaultPath: string,
): PathPickerOptions {
  return {
    directory: target === "skills",
    multiple: false,
    defaultPath,
  };
}

export function pickPathValueFromDialogResult(
  result: string | string[] | null,
): string | null {
  return typeof result === "string" ? result : null;
}

export function updateDraftWithPickedPath(
  draft: ToolPathDraft,
  target: PathPickerTarget,
  selectedPath: string | null,
): ToolPathDraft {
  if (!selectedPath) {
    return draft;
  }
  if (target === "skills") {
    return {
      ...draft,
      skillsDir: selectedPath,
    };
  }
  return {
    ...draft,
    rulesPath: selectedPath,
  };
}
