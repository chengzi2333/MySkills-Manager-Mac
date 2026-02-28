import type { ToolStatus } from "../api/tauri";

export type ToolPathDiagnostics = {
  skillsPathHealthy: boolean;
  rulesPathHealthy: boolean;
  skillsPathLabel: string;
  rulesPathLabel: string;
};

export function buildToolPathDiagnostics(tool: ToolStatus): ToolPathDiagnostics {
  const rulesPathConfigured = tool.rulesPath.trim().length > 0;
  const skillsPathHealthy = tool.skillsDirExists && tool.skillsDirWritable;
  const rulesPathHealthy =
    !rulesPathConfigured || (tool.rulesPathExists && tool.rulesPathWritable);
  const skillsPathLabel = !tool.skillsDirExists
    ? "Skills Path Missing"
    : tool.skillsDirWritable
      ? "Skills Path OK"
      : "Skills Path Read-only";
  const rulesPathLabel = !rulesPathConfigured
    ? "Rules Path Unset"
    : !tool.rulesPathExists
      ? "Rules Path Missing"
      : tool.rulesPathWritable
        ? "Rules Path OK"
        : "Rules Path Read-only";

  return {
    skillsPathHealthy,
    rulesPathHealthy,
    skillsPathLabel,
    rulesPathLabel,
  };
}
