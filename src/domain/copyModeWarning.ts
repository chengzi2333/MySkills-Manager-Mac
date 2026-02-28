type CopyModeTool = {
  name: string;
  exists: boolean;
  syncMode: string;
};

export function copyModeToolsRequiringResync(tools: CopyModeTool[]): string[] {
  return tools
    .filter((tool) => tool.exists && tool.syncMode.toLowerCase() === "copy")
    .map((tool) => tool.name);
}
