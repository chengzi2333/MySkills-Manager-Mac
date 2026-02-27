import { invoke } from "@tauri-apps/api/core";

export const APP_ERROR_EVENT = "myskills:error";

function normalizeInvokeError(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message;
  }
  return String(error);
}

function reportGlobalError(message: string) {
  if (typeof window !== "undefined") {
    window.dispatchEvent(new CustomEvent(APP_ERROR_EVENT, { detail: message }));
  }
}

async function invokeWithError<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error: unknown) {
    const message = normalizeInvokeError(error);
    reportGlobalError(message);
    throw new Error(message);
  }
}

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

export type OnboardingState = {
  completed: boolean;
  skillsDir: string;
  autoSync: boolean;
};

export type OnboardingSetSkillsDirResult = {
  success: boolean;
  skills: SkillMeta[];
};

export type OnboardingCompleteResult = {
  success: boolean;
  autoSync: boolean;
  configuredTools: number;
};

export async function appPing(): Promise<string> {
  return invokeWithError<string>("app_ping");
}

export async function skillsList(): Promise<SkillMeta[]> {
  return invokeWithError<SkillMeta[]>("skills_list");
}

export async function skillsGetContent(name: string): Promise<SkillDocument> {
  return invokeWithError<SkillDocument>("skills_get_content", { name });
}

export async function skillsSaveContent(
  name: string,
  content: string,
): Promise<SaveResult> {
  return invokeWithError<SaveResult>("skills_save_content", { name, content });
}

export async function statsGet(days?: number): Promise<StatsResult> {
  return invokeWithError<StatsResult>("stats_get", { days });
}

export async function logsGet(query: LogsQuery): Promise<LogsResult> {
  return invokeWithError<LogsResult>("logs_get", query);
}

export async function rulesGet(): Promise<RulesContent> {
  return invokeWithError<RulesContent>("rules_get");
}

export async function rulesSave(content: string): Promise<RulesSaveResult> {
  return invokeWithError<RulesSaveResult>("rules_save", { content });
}

export async function gitStatus(): Promise<GitStatus> {
  return invokeWithError<GitStatus>("git_status");
}

export async function gitCommit(message: string): Promise<GitCommitResult> {
  return invokeWithError<GitCommitResult>("git_commit", { message });
}

export async function gitPush(): Promise<GitPushResult> {
  return invokeWithError<GitPushResult>("git_push");
}

export async function setupStatus(): Promise<ToolStatus[]> {
  return invokeWithError<ToolStatus[]>("setup_status");
}

export async function setupApply(
  tools: string[],
  skills?: SkillSyncConfig[],
): Promise<SetupApplyResult[]> {
  return invokeWithError<SetupApplyResult[]>("setup_apply", { tools, skills });
}

export async function setupGetCustomTools(): Promise<CustomTool[]> {
  return invokeWithError<CustomTool[]>("setup_get_custom_tools");
}

export async function setupAddCustomTool(tool: CustomTool): Promise<SetupMutationResult> {
  return invokeWithError<SetupMutationResult>("setup_add_custom_tool", {
    name: tool.name,
    id: tool.id,
    skills_dir: tool.skillsDir,
    rules_file: tool.rulesFile,
    icon: tool.icon,
  });
}

export async function setupRemoveCustomTool(id: string): Promise<SetupMutationResult> {
  return invokeWithError<SetupMutationResult>("setup_remove_custom_tool", { id });
}

export async function onboardingGetState(): Promise<OnboardingState> {
  return invokeWithError<OnboardingState>("onboarding_get_state");
}

export async function onboardingSetSkillsDir(
  dir: string,
  createIfMissing = false,
): Promise<OnboardingSetSkillsDirResult> {
  return invokeWithError<OnboardingSetSkillsDirResult>("onboarding_set_skills_dir", {
    dir,
    create_if_missing: createIfMissing,
  });
}

export async function onboardingComplete(autoSync: boolean): Promise<OnboardingCompleteResult> {
  return invokeWithError<OnboardingCompleteResult>("onboarding_complete", { auto_sync: autoSync });
}
