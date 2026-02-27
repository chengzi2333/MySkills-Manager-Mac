import { invoke } from "@tauri-apps/api/core";

export type SkillMeta = {
  name: string;
  description?: string;
  category?: string;
  tags?: string[];
  my_notes?: string;
  last_updated?: string;
  directory: string;
};

export type SkillDocument = {
  frontmatter: Record<string, unknown>;
  body: string;
};

export type SaveResult = {
  success: boolean;
};

export type RulesContent = {
  content: string;
};

export type RulesSaveResult = {
  success: boolean;
};

export type LogEntry = {
  ts: string;
  skill: string;
  cwd: string;
  tool: string;
  session?: string;
};

export type LogsResult = {
  logs: LogEntry[];
  total: number;
};

export type NamedCount = {
  name: string;
  count: number;
};

export type DayCount = {
  date: string;
  count: number;
};

export type StatsResult = {
  total_invocations: number;
  by_skill: NamedCount[];
  by_tool: NamedCount[];
  by_day: DayCount[];
  recent: LogEntry[];
  unused_skills: string[];
};

export type LogsQuery = {
  skill?: string;
  tool?: string;
  from?: string;
  to?: string;
  page?: number;
  limit?: number;
};

export type GitStatus = {
  branch: string;
  changed: string[];
  staged: string[];
  not_added: string[];
  ahead: number;
  behind: number;
};

export type GitCommitResult = {
  success: boolean;
  hash: string;
};

export type GitPushResult = {
  success: boolean;
  error?: string;
};

export type ToolStatus = {
  name: string;
  id: string;
  skillsDir: string;
  rulesPath: string;
  exists: boolean;
  configured: boolean;
  syncedSkills: number;
  syncMode: "symlink" | "copy" | "none" | string;
  hookConfigured: boolean;
  isCustom: boolean;
};

export type SetupApplyResult = {
  tool: string;
  success: boolean;
  action: string;
  syncMode: string;
  syncedCount: number;
  error?: string;
};

export type SkillSyncConfig = {
  skillName: string;
  enabledTools: string[];
};

export type CustomTool = {
  name: string;
  id: string;
  skillsDir: string;
  rulesFile?: string;
  icon?: string;
};

export type SetupMutationResult = {
  success: boolean;
};

export async function appPing(): Promise<string> {
  return invoke<string>("app_ping");
}

export async function skillsList(): Promise<SkillMeta[]> {
  return invoke<SkillMeta[]>("skills_list");
}

export async function skillsGetContent(name: string): Promise<SkillDocument> {
  return invoke<SkillDocument>("skills_get_content", { name });
}

export async function skillsSaveContent(
  name: string,
  content: string,
): Promise<SaveResult> {
  return invoke<SaveResult>("skills_save_content", { name, content });
}

export async function statsGet(days?: number): Promise<StatsResult> {
  return invoke<StatsResult>("stats_get", { days });
}

export async function logsGet(query: LogsQuery): Promise<LogsResult> {
  return invoke<LogsResult>("logs_get", query);
}

export async function rulesGet(): Promise<RulesContent> {
  return invoke<RulesContent>("rules_get");
}

export async function rulesSave(content: string): Promise<RulesSaveResult> {
  return invoke<RulesSaveResult>("rules_save", { content });
}

export async function gitStatus(): Promise<GitStatus> {
  return invoke<GitStatus>("git_status");
}

export async function gitCommit(message: string): Promise<GitCommitResult> {
  return invoke<GitCommitResult>("git_commit", { message });
}

export async function gitPush(): Promise<GitPushResult> {
  return invoke<GitPushResult>("git_push");
}

export async function setupStatus(): Promise<ToolStatus[]> {
  return invoke<ToolStatus[]>("setup_status");
}

export async function setupApply(
  tools: string[],
  skills?: SkillSyncConfig[],
): Promise<SetupApplyResult[]> {
  return invoke<SetupApplyResult[]>("setup_apply", { tools, skills });
}

export async function setupGetCustomTools(): Promise<CustomTool[]> {
  return invoke<CustomTool[]>("setup_get_custom_tools");
}

export async function setupAddCustomTool(tool: CustomTool): Promise<SetupMutationResult> {
  return invoke<SetupMutationResult>("setup_add_custom_tool", {
    name: tool.name,
    id: tool.id,
    skillsDir: tool.skillsDir,
    rulesFile: tool.rulesFile,
    icon: tool.icon,
  });
}

export async function setupRemoveCustomTool(id: string): Promise<SetupMutationResult> {
  return invoke<SetupMutationResult>("setup_remove_custom_tool", { id });
}
