import { useEffect, useMemo, useState } from "react";

import {
  setupAddCustomTool,
  setupApply,
  setupRemoveCustomTool,
  setupStatus,
  type SetupApplyResult,
  type ToolStatus,
} from "../api/tauri";
import "./ToolsPage.css";

type CustomToolForm = {
  name: string;
  id: string;
  skillsDir: string;
  rulesFile: string;
  icon: string;
};

const EMPTY_FORM: CustomToolForm = {
  name: "",
  id: "",
  skillsDir: "",
  rulesFile: "",
  icon: "",
};

export default function ToolsPage() {
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [selected, setSelected] = useState<Record<string, boolean>>({});
  const [status, setStatus] = useState("");
  const [applyResults, setApplyResults] = useState<SetupApplyResult[]>([]);
  const [form, setForm] = useState<CustomToolForm>(EMPTY_FORM);
  const [busy, setBusy] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  async function loadStatus() {
    setBusy(true);
    setStatus("loading...");
    try {
      const data = await setupStatus();
      setTools(data);
      setSelected((prev) => {
        const next: Record<string, boolean> = {};
        for (const tool of data) {
          next[tool.id] = prev[tool.id] ?? !tool.isCustom;
        }
        return next;
      });
      setStatus("");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    void loadStatus();
  }, []);

  const selectedToolIds = useMemo(
    () => tools.filter((tool) => selected[tool.id]).map((tool) => tool.id),
    [tools, selected],
  );

  async function handleApply() {
    if (selectedToolIds.length === 0) {
      setStatus("Please select at least one tool.");
      return;
    }
    setBusy(true);
    setStatus("syncing...");
    try {
      const results = await setupApply(selectedToolIds);
      setApplyResults(results);
      await loadStatus();
      setStatus("");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleAddCustomTool() {
    if (!form.name.trim() || !form.id.trim() || !form.skillsDir.trim()) {
      setStatus("name / id / skillsDir are required.");
      return;
    }

    setSubmitting(true);
    setStatus("adding custom tool...");
    try {
      await setupAddCustomTool({
        name: form.name.trim(),
        id: form.id.trim(),
        skillsDir: form.skillsDir.trim(),
        rulesFile: form.rulesFile.trim() || undefined,
        icon: form.icon.trim() || undefined,
      });
      setForm(EMPTY_FORM);
      await loadStatus();
      setStatus("custom tool added");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleRemoveCustomTool(id: string) {
    setBusy(true);
    setStatus("removing custom tool...");
    try {
      await setupRemoveCustomTool(id);
      await loadStatus();
      setStatus("custom tool removed");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="page animate-fadein">
      <header className="page-header">
        <h1 className="page-title">Tools</h1>
        <div className="tools-header-actions">
          <span className="page-count">{status || `${tools.length} tools`}</span>
          <button
            className="btn btn-ghost"
            onClick={() => void loadStatus()}
            disabled={busy}
          >
            Refresh
          </button>
          <button
            className="btn btn-primary"
            onClick={() => void handleApply()}
            disabled={busy || selectedToolIds.length === 0}
          >
            {busy ? "Syncing..." : `Apply to ${selectedToolIds.length} tool(s)`}
          </button>
        </div>
      </header>

      <div className="tools-grid">
        {tools.map((tool) => (
          <article key={tool.id} className="tools-card">
            <header className="tools-card-header">
              <h3 className="tools-card-title">
                {tool.name}
                {tool.isCustom && <span className="tools-chip">Custom</span>}
              </h3>
              <label className="tools-toggle">
                <input
                  type="checkbox"
                  checked={Boolean(selected[tool.id])}
                  onChange={(e) =>
                    setSelected((prev) => ({ ...prev, [tool.id]: e.target.checked }))
                  }
                />
                <span>Select</span>
              </label>
            </header>

            <dl className="tools-metrics">
              <div>
                <dt>Synced Skills</dt>
                <dd>{tool.syncedSkills}</dd>
              </div>
              <div>
                <dt>Sync Mode</dt>
                <dd>{tool.syncMode}</dd>
              </div>
            </dl>

            <div className="tools-flags">
              <span className={`tools-flag ${tool.exists ? "ok" : "warn"}`}>
                {tool.exists ? "Tool Exists" : "Tool Missing"}
              </span>
              <span className={`tools-flag ${tool.configured ? "ok" : "warn"}`}>
                {tool.configured ? "Rules Injected" : "Rules Missing"}
              </span>
              {tool.id === "claude-code" && (
                <span className={`tools-flag ${tool.hookConfigured ? "ok" : "warn"}`}>
                  {tool.hookConfigured ? "Hook OK" : "Hook Missing"}
                </span>
              )}
            </div>

            <p className="tools-path">
              <strong>Skills:</strong> {tool.skillsDir}
            </p>
            {tool.rulesPath && (
              <p className="tools-path">
                <strong>Rules:</strong> {tool.rulesPath}
              </p>
            )}

            {tool.isCustom && (
              <button
                className="btn btn-ghost"
                onClick={() => void handleRemoveCustomTool(tool.id)}
                disabled={busy}
              >
                Remove
              </button>
            )}
          </article>
        ))}
      </div>

      <article className="chart-card tools-form-card">
        <h3 className="chart-title">Add Custom Tool</h3>
        <div className="tools-form-grid">
          <label className="field">
            <span className="field-label">Name *</span>
            <input
              className="field-input"
              value={form.name}
              onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
              placeholder="Aider"
            />
          </label>
          <label className="field">
            <span className="field-label">ID *</span>
            <input
              className="field-input"
              value={form.id}
              onChange={(e) => setForm((prev) => ({ ...prev, id: e.target.value }))}
              placeholder="aider"
            />
          </label>
          <label className="field field-wide">
            <span className="field-label">Skills Directory *</span>
            <input
              className="field-input"
              value={form.skillsDir}
              onChange={(e) => setForm((prev) => ({ ...prev, skillsDir: e.target.value }))}
              placeholder="C:\\Users\\Keith\\.aider\\skills"
            />
          </label>
          <label className="field field-wide">
            <span className="field-label">Rules File</span>
            <input
              className="field-input"
              value={form.rulesFile}
              onChange={(e) => setForm((prev) => ({ ...prev, rulesFile: e.target.value }))}
              placeholder="C:\\Users\\Keith\\.aider\\AGENTS.md"
            />
          </label>
          <label className="field">
            <span className="field-label">Icon</span>
            <input
              className="field-input"
              value={form.icon}
              onChange={(e) => setForm((prev) => ({ ...prev, icon: e.target.value }))}
              placeholder="optional"
            />
          </label>
        </div>
        <div className="tools-form-actions">
          <button
            className="btn btn-primary"
            onClick={() => void handleAddCustomTool()}
            disabled={submitting}
          >
            {submitting ? "Adding..." : "Add Tool"}
          </button>
        </div>
      </article>

      {applyResults.length > 0 && (
        <article className="chart-card">
          <h3 className="chart-title">Last Apply Result</h3>
          <div className="tools-results">
            {applyResults.map((result) => (
              <div key={result.tool} className="tools-result-item">
                <strong>{result.tool}</strong>
                <span className={`tools-flag ${result.success ? "ok" : "warn"}`}>
                  {result.success ? "Success" : "Failed"}
                </span>
                <span>{result.action}</span>
                {result.error && <span className="tools-error">{result.error}</span>}
              </div>
            ))}
          </div>
        </article>
      )}
    </div>
  );
}
