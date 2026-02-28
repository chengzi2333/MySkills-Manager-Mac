import { useCallback, useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import {
  setupAddCustomTool,
  setupApply,
  setupRemoveCustomTool,
  setupSetToolAutoSync,
  setupSetToolTrackingEnabled,
  setupStatus,
  setupUpdateToolPaths,
  type SetupApplyResult,
  type ToolStatus,
} from "../api/tauri";
import { IconClose, IconFolder, IconPlus, IconRefresh, IconSave } from "../components/icons";
import ToolLogo from "../components/ToolLogo";
import { useI18n } from "../i18n/I18nProvider";
import {
  buildPathPickerOptions,
  pickPathValueFromDialogResult,
  updateDraftWithPickedPath,
  type PathPickerTarget,
  type ToolPathDraft,
} from "./toolsPathPicker";
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
  const { t } = useI18n();
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [pathDrafts, setPathDrafts] = useState<Record<string, ToolPathDraft>>({});
  const [status, setStatus] = useState("");
  const [applyResults, setApplyResults] = useState<SetupApplyResult[]>([]);
  const [form, setForm] = useState<CustomToolForm>(EMPTY_FORM);
  const [busy, setBusy] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [savingPathToolId, setSavingPathToolId] = useState<string | null>(null);
  const [syncingToolId, setSyncingToolId] = useState<string | null>(null);
  const [togglingAutoToolId, setTogglingAutoToolId] = useState<string | null>(null);
  const [togglingTrackingToolId, setTogglingTrackingToolId] = useState<string | null>(null);
  const [showCustomForm, setShowCustomForm] = useState(false);

  const loadStatus = useCallback(async () => {
    setBusy(true);
    setStatus(t("tools.loading"));
    try {
      const data = await setupStatus();
      setTools(data);
      setPathDrafts(() => {
        const next: Record<string, ToolPathDraft> = {};
        for (const tool of data) {
          next[tool.id] = {
            skillsDir: tool.skillsDir,
            rulesPath: tool.rulesPath ?? "",
          };
        }
        return next;
      });
      setStatus("");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  }, [t]);

  useEffect(() => {
    void loadStatus();
  }, [loadStatus]);

  const installedTools = useMemo(() => tools.filter((tool) => tool.exists), [tools]);
  const uninstalledTools = useMemo(() => tools.filter((tool) => !tool.exists), [tools]);
  const autoToolIds = useMemo(
    () => tools.filter((tool) => tool.exists && tool.autoSync).map((tool) => tool.id),
    [tools],
  );

  function normalizedPath(value: string | undefined) {
    return (value ?? "").trim();
  }

  async function handleApplyAutoTools() {
    if (autoToolIds.length === 0) {
      setStatus(t("tools.apply.auto.none"));
      return;
    }
    setBusy(true);
    setStatus(t("tools.syncing"));
    try {
      const results = await setupApply(autoToolIds);
      setApplyResults(results);
      await loadStatus();
      setStatus("");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleManualSync(tool: ToolStatus) {
    setSyncingToolId(tool.id);
    setStatus(t("tools.syncing"));
    try {
      const results = await setupApply([tool.id]);
      setApplyResults(results);
      await loadStatus();
      setStatus("");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setSyncingToolId(null);
    }
  }

  async function handleToggleAutoSync(tool: ToolStatus) {
    setTogglingAutoToolId(tool.id);
    setStatus(t("tools.auto.updating"));
    try {
      await setupSetToolAutoSync(tool.id, !tool.autoSync);
      setTools((prev) =>
        prev.map((item) =>
          item.id === tool.id ? { ...item, autoSync: !item.autoSync } : item,
        ),
      );
      setStatus("");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setTogglingAutoToolId(null);
    }
  }

  async function handleToggleTracking(tool: ToolStatus) {
    setTogglingTrackingToolId(tool.id);
    setStatus(t("tools.tracking.updating"));
    try {
      await setupSetToolTrackingEnabled(tool.id, !tool.trackingEnabled);
      setTools((prev) =>
        prev.map((item) =>
          item.id === tool.id ? { ...item, trackingEnabled: !item.trackingEnabled } : item,
        ),
      );
      setStatus("");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setTogglingTrackingToolId(null);
    }
  }

  async function handleAddCustomTool() {
    if (!form.name.trim() || !form.id.trim() || !form.skillsDir.trim()) {
      setStatus(t("tools.validation.required"));
      return;
    }

    setSubmitting(true);
    setStatus(t("tools.custom.adding"));
    try {
      await setupAddCustomTool({
        name: form.name.trim(),
        id: form.id.trim(),
        skillsDir: form.skillsDir.trim(),
        rulesFile: form.rulesFile.trim() || undefined,
        icon: form.icon.trim() || undefined,
      });
      setForm(EMPTY_FORM);
      setShowCustomForm(false);
      await loadStatus();
      setStatus(t("tools.custom.added"));
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleRemoveCustomTool(id: string) {
    setBusy(true);
    setStatus(t("tools.custom.removing"));
    try {
      await setupRemoveCustomTool(id);
      await loadStatus();
      setStatus(t("tools.custom.removed"));
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleSaveToolPaths(tool: ToolStatus) {
    const draft = pathDrafts[tool.id];
    if (!draft || !normalizedPath(draft.skillsDir)) {
      setStatus(t("tools.validation.skillsRequired"));
      return;
    }

    setSavingPathToolId(tool.id);
    setStatus(t("tools.path.saving"));
    try {
      await setupUpdateToolPaths(
        tool.id,
        normalizedPath(draft.skillsDir),
        normalizedPath(draft.rulesPath) || undefined,
      );
      await loadStatus();
      setStatus(t("tools.path.saved"));
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setSavingPathToolId(null);
    }
  }

  async function handlePickToolPath(toolId: string, target: PathPickerTarget) {
    const draft = pathDrafts[toolId];
    if (!draft) {
      return;
    }

    const selectedResult = await open({
      ...buildPathPickerOptions(
        target,
        target === "skills" ? draft.skillsDir : draft.rulesPath,
      ),
      title: target === "skills" ? t("tools.path.pickDir") : t("tools.path.pickFile"),
    });

    const selectedPath = pickPathValueFromDialogResult(selectedResult);
    setPathDrafts((prev) => {
      const currentDraft = prev[toolId];
      if (!currentDraft) {
        return prev;
      }
      return {
        ...prev,
        [toolId]: updateDraftWithPickedPath(currentDraft, target, selectedPath),
      };
    });
  }

  async function handlePickCustomFormPath(target: PathPickerTarget) {
    const selectedResult = await open({
      ...buildPathPickerOptions(
        target,
        target === "skills" ? form.skillsDir : form.rulesFile,
      ),
      title: target === "skills" ? t("tools.path.pickDir") : t("tools.path.pickFile"),
    });
    const selectedPath = pickPathValueFromDialogResult(selectedResult);
    if (!selectedPath) {
      return;
    }
    if (target === "skills") {
      setForm((prev) => ({ ...prev, skillsDir: selectedPath }));
      return;
    }
    setForm((prev) => ({ ...prev, rulesFile: selectedPath }));
  }

  function renderToolCard(tool: ToolStatus, installed: boolean) {
    const draft = pathDrafts[tool.id] ?? {
      skillsDir: tool.skillsDir,
      rulesPath: tool.rulesPath ?? "",
    };
    const hasPathChange =
      normalizedPath(draft.skillsDir) !== normalizedPath(tool.skillsDir) ||
      normalizedPath(draft.rulesPath) !== normalizedPath(tool.rulesPath);
    const savingCurrentTool = savingPathToolId === tool.id;
    const syncingCurrentTool = syncingToolId === tool.id;
    const togglingCurrentTool = togglingAutoToolId === tool.id;
    const togglingTrackingCurrentTool = togglingTrackingToolId === tool.id;

    return (
      <article key={tool.id} className={`tool-card ${installed ? "" : "tool-card-uninstalled"}`.trim()}>
        <header className="tool-card-header">
          <div className="tool-card-identity">
            <ToolLogo id={tool.id} name={tool.name} icon={tool.icon} />
            <div className="tool-card-title-wrap">
              <div className="tool-card-title-row">
                <h3 className="tool-card-title">{tool.name}</h3>
                <span className={`tool-card-badge ${installed ? "ok" : "neutral"}`}>
                  {installed ? t("tools.status.detected") : t("tools.status.notDetected")}
                </span>
                {tool.syncMode !== "none" && (
                  <span className="tool-card-badge neutral">{tool.syncMode.toUpperCase()}</span>
                )}
                {tool.isCustom && <span className="tool-card-badge neutral">{t("tools.custom")}</span>}
              </div>
              <p className="tool-card-id">ID: {tool.id}</p>
            </div>
          </div>
          <div className="tool-card-toggles">
            <div className="tool-card-toggle-wrap">
              <span className="tool-switch-label">
                {t("tools.auto.toggle")}
              </span>
              <button
                type="button"
                className={`tool-switch ${tool.autoSync ? "active" : ""}`}
                aria-pressed={tool.autoSync}
                aria-label={`${tool.name} auto sync toggle`}
                onClick={() => void handleToggleAutoSync(tool)}
                disabled={busy || togglingCurrentTool}
              >
                <span className="tool-switch-thumb" />
              </button>
            </div>
            <div className="tool-card-toggle-wrap">
              <span className="tool-switch-label">
                {t("tools.tracking.toggle")}
              </span>
              <button
                type="button"
                className={`tool-switch ${tool.trackingEnabled ? "active" : ""}`}
                aria-pressed={tool.trackingEnabled}
                aria-label={`${tool.name} tracking toggle`}
                onClick={() => void handleToggleTracking(tool)}
                disabled={busy || togglingTrackingCurrentTool}
              >
                <span className="tool-switch-thumb" />
              </button>
            </div>
          </div>
        </header>

        <div className="tool-card-divider" />

        <details className="tool-paths-accordion">
          <summary className="tool-paths-summary">{t("tools.path.section")}</summary>
          <div className="tool-card-paths">
            <label className="tool-path-field">
              <span className="tool-path-label">{t("tools.path.rules")}</span>
              <div className="tool-path-row">
                <input
                  className="field-input tools-path-input"
                  value={draft.rulesPath}
                  onChange={(e) =>
                    setPathDrafts((prev) => ({
                      ...prev,
                      [tool.id]: { ...draft, rulesPath: e.target.value },
                    }))
                  }
                  placeholder="C:\\Users\\Keith\\.codex\\AGENTS.md"
                />
                <button
                  type="button"
                  className="tool-path-picker"
                  onClick={() => void handlePickToolPath(tool.id, "rules")}
                  disabled={busy || savingCurrentTool}
                  title={t("tools.path.pickFile")}
                  aria-label={t("tools.path.pickFile")}
                >
                  <IconFolder size={15} />
                </button>
              </div>
            </label>

            <label className="tool-path-field">
              <span className="tool-path-label">{t("tools.path.skills")}</span>
              <div className="tool-path-row">
                <input
                  className="field-input tools-path-input"
                  value={draft.skillsDir}
                  onChange={(e) =>
                    setPathDrafts((prev) => ({
                      ...prev,
                      [tool.id]: { ...draft, skillsDir: e.target.value },
                    }))
                  }
                  placeholder="C:\\Users\\Keith\\.codex\\skills"
                />
                <button
                  type="button"
                  className="tool-path-picker"
                  onClick={() => void handlePickToolPath(tool.id, "skills")}
                  disabled={busy || savingCurrentTool}
                  title={t("tools.path.pickDir")}
                  aria-label={t("tools.path.pickDir")}
                >
                  <IconFolder size={15} />
                </button>
              </div>
            </label>
          </div>
        </details>

        <footer className="tool-card-footer">
          <div className="tool-card-meta">
            <span>{t("tools.syncedSkills")}: {tool.syncedSkills}</span>
            <span>{t("tools.syncMode")}: {tool.syncMode}</span>
          </div>

          <div className="tool-card-flags">
            <span className={`tool-card-badge ${tool.configured ? "ok" : "warn"}`}>
              {tool.configured ? t("tools.flag.rulesOk") : t("tools.flag.rulesMissing")}
            </span>
            {tool.id === "claude-code" && (
              <span className={`tool-card-badge ${tool.hookConfigured ? "ok" : "warn"}`}>
                {tool.hookConfigured ? t("tools.flag.hookOk") : t("tools.flag.hookMissing")}
              </span>
            )}
          </div>
        </footer>

        <div className="tool-card-actions">
          {installed && (
            <button
              type="button"
              className="btn btn-primary tool-card-action-btn"
              onClick={() => void handleManualSync(tool)}
              disabled={busy || syncingCurrentTool}
            >
              {syncingCurrentTool ? t("tools.manual.syncing") : t("tools.manual.button")}
            </button>
          )}
          <button
            type="button"
            className="btn btn-ghost tool-card-action-btn"
            onClick={() => void handleSaveToolPaths(tool)}
            disabled={busy || savingCurrentTool || !hasPathChange || !normalizedPath(draft.skillsDir)}
          >
            {savingCurrentTool ? t("tools.path.saving") : t("tools.path.save")}
          </button>
          {tool.isCustom && (
            <button
              className="btn btn-ghost tool-card-action-btn"
              onClick={() => void handleRemoveCustomTool(tool.id)}
              disabled={busy || savingCurrentTool}
            >
              {t("tools.remove")}
            </button>
          )}
        </div>
      </article>
    );
  }

  return (
    <div className="page animate-fadein tools-page">
      <header className="page-header tools-page-header">
        <div className="tools-header-copy">
          <h1 className="page-title">{t("tools.title")}</h1>
          <p className="tools-installed">{t("tools.count", { count: tools.length })}</p>
        </div>
        <div className="tools-header-actions">
          {status && (
            <span className="tools-header-status" role="status" aria-live="polite">
              {status}
            </span>
          )}
          <button className="btn btn-ghost tools-header-btn" onClick={() => void loadStatus()} disabled={busy}>
            <IconRefresh size={14} />
            {t("tools.refresh")}
          </button>
          <button
            className={`btn tools-header-btn ${showCustomForm ? "btn-ghost" : "btn-primary"}`}
            onClick={() => setShowCustomForm((prev) => !prev)}
            disabled={submitting}
          >
            {showCustomForm ? <IconClose size={14} /> : <IconPlus size={14} />}
            {showCustomForm ? t("tools.form.hide") : t("tools.form.show")}
          </button>
          <button
            className="btn btn-primary tools-header-btn"
            onClick={() => void handleApplyAutoTools()}
            disabled={busy || autoToolIds.length === 0}
          >
            <IconSave size={14} />
            {busy ? t("tools.syncing") : t("tools.apply.auto.button", { count: autoToolIds.length })}
          </button>
        </div>
      </header>

      <div className="tools-sections">
        <section className="tools-section">
          <h2 className="tools-section-title">{t("tools.section.installed", { count: installedTools.length })}</h2>
          <div className="tools-grid">{installedTools.map((tool) => renderToolCard(tool, true))}</div>
        </section>

        <section className="tools-section">
          <h2 className="tools-section-title">{t("tools.section.uninstalled", { count: uninstalledTools.length })}</h2>
          <div className="tools-grid">{uninstalledTools.map((tool) => renderToolCard(tool, false))}</div>
        </section>
      </div>

      {showCustomForm && (
        <article className="chart-card tools-form-card">
          <header className="tools-form-head">
            <h3 className="chart-title">{t("tools.form.title")}</h3>
            <button className="btn btn-ghost tools-form-hide-btn" onClick={() => setShowCustomForm(false)}>
              <IconClose size={14} />
              {t("tools.form.hide")}
            </button>
          </header>
          <div className="tools-form-grid">
            <label className="field">
              <span className="field-label">{t("tools.form.name")}</span>
              <input
                className="field-input"
                value={form.name}
                onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                placeholder="Aider"
              />
            </label>
            <label className="field">
              <span className="field-label">{t("tools.form.id")}</span>
              <input
                className="field-input"
                value={form.id}
                onChange={(e) => setForm((prev) => ({ ...prev, id: e.target.value }))}
                placeholder="aider"
              />
            </label>
            <label className="field field-wide">
              <span className="field-label">{t("tools.form.skillsDir")}</span>
              <div className="tool-path-row">
                <input
                  className="field-input tools-path-input"
                  value={form.skillsDir}
                  onChange={(e) => setForm((prev) => ({ ...prev, skillsDir: e.target.value }))}
                  placeholder="C:\\Users\\Keith\\.aider\\skills"
                />
                <button
                  type="button"
                  className="tool-path-picker"
                  onClick={() => void handlePickCustomFormPath("skills")}
                  disabled={submitting}
                  title={t("tools.path.pickDir")}
                  aria-label={t("tools.path.pickDir")}
                >
                  <IconFolder size={15} />
                </button>
              </div>
            </label>
            <label className="field field-wide">
              <span className="field-label">{t("tools.form.rulesFile")}</span>
              <div className="tool-path-row">
                <input
                  className="field-input tools-path-input"
                  value={form.rulesFile}
                  onChange={(e) => setForm((prev) => ({ ...prev, rulesFile: e.target.value }))}
                  placeholder="C:\\Users\\Keith\\.aider\\AGENTS.md"
                />
                <button
                  type="button"
                  className="tool-path-picker"
                  onClick={() => void handlePickCustomFormPath("rules")}
                  disabled={submitting}
                  title={t("tools.path.pickFile")}
                  aria-label={t("tools.path.pickFile")}
                >
                  <IconFolder size={15} />
                </button>
              </div>
            </label>
            <label className="field">
              <span className="field-label">{t("tools.form.icon")}</span>
              <input
                className="field-input"
                value={form.icon}
                onChange={(e) => setForm((prev) => ({ ...prev, icon: e.target.value }))}
                placeholder="openai | anthropic | /tool-logos/custom.svg"
              />
            </label>
          </div>
          <div className="tools-form-actions">
            <button className="btn btn-primary" onClick={() => void handleAddCustomTool()} disabled={submitting}>
              {submitting ? t("tools.form.adding") : t("tools.form.add")}
            </button>
          </div>
        </article>
      )}

      {applyResults.length > 0 && (
        <article className="chart-card">
          <h3 className="chart-title">{t("tools.result.title")}</h3>
          <div className="tools-results">
            {applyResults.map((result) => (
              <div key={result.tool} className="tools-result-item">
                <strong>{result.tool}</strong>
                <span className={`tool-card-badge ${result.success ? "ok" : "warn"}`}>
                  {result.success ? t("tools.result.success") : t("tools.result.failed")}
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

