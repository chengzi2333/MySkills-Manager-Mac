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
import { IconClose, IconPlus, IconRefresh, IconSave } from "../components/icons";
import { useI18n } from "../i18n/I18nProvider";
import {
  buildPathPickerOptions,
  pickPathValueFromDialogResult,
  updateDraftWithPickedPath,
  type PathPickerTarget,
  type ToolPathDraft,
} from "./toolsPathPicker";
import ToolCard from "./tools/ToolCard";
import CustomToolFormCard from "./tools/CustomToolFormCard";
import { EMPTY_CUSTOM_TOOL_FORM, type CustomToolForm } from "./tools/customToolForm";
import "./ToolsPage.css";

export default function ToolsPage() {
  const { t, locale } = useI18n();
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [pathDrafts, setPathDrafts] = useState<Record<string, ToolPathDraft>>({});
  const [status, setStatus] = useState("");
  const [applyResults, setApplyResults] = useState<SetupApplyResult[]>([]);
  const [form, setForm] = useState<CustomToolForm>(EMPTY_CUSTOM_TOOL_FORM);
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
      setForm(EMPTY_CUSTOM_TOOL_FORM);
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
          <div className="tools-grid">
            {installedTools.map((tool) => {
              const draft = pathDrafts[tool.id] ?? {
                skillsDir: tool.skillsDir,
                rulesPath: tool.rulesPath ?? "",
              };
              return (
                <ToolCard
                  key={tool.id}
                  tool={tool}
                  installed
                  draft={draft}
                  hasPathChange={
                    normalizedPath(draft.skillsDir) !== normalizedPath(tool.skillsDir) ||
                    normalizedPath(draft.rulesPath) !== normalizedPath(tool.rulesPath)
                  }
                  busy={busy}
                  savingCurrentTool={savingPathToolId === tool.id}
                  syncingCurrentTool={syncingToolId === tool.id}
                  togglingAutoCurrentTool={togglingAutoToolId === tool.id}
                  togglingTrackingCurrentTool={togglingTrackingToolId === tool.id}
                  skillsDirConfigured={Boolean(normalizedPath(draft.skillsDir))}
                  locale={locale}
                  t={t}
                  onDraftChange={(nextDraft) =>
                    setPathDrafts((prev) => ({ ...prev, [tool.id]: nextDraft }))
                  }
                  onPickToolPath={(target) => void handlePickToolPath(tool.id, target)}
                  onManualSync={() => void handleManualSync(tool)}
                  onToggleAutoSync={() => void handleToggleAutoSync(tool)}
                  onToggleTracking={() => void handleToggleTracking(tool)}
                  onSaveToolPaths={() => void handleSaveToolPaths(tool)}
                  onRemoveCustomTool={() => void handleRemoveCustomTool(tool.id)}
                />
              );
            })}
          </div>
        </section>

        <section className="tools-section">
          <h2 className="tools-section-title">{t("tools.section.uninstalled", { count: uninstalledTools.length })}</h2>
          <div className="tools-grid">
            {uninstalledTools.map((tool) => {
              const draft = pathDrafts[tool.id] ?? {
                skillsDir: tool.skillsDir,
                rulesPath: tool.rulesPath ?? "",
              };
              return (
                <ToolCard
                  key={tool.id}
                  tool={tool}
                  installed={false}
                  draft={draft}
                  hasPathChange={
                    normalizedPath(draft.skillsDir) !== normalizedPath(tool.skillsDir) ||
                    normalizedPath(draft.rulesPath) !== normalizedPath(tool.rulesPath)
                  }
                  busy={busy}
                  savingCurrentTool={savingPathToolId === tool.id}
                  syncingCurrentTool={syncingToolId === tool.id}
                  togglingAutoCurrentTool={togglingAutoToolId === tool.id}
                  togglingTrackingCurrentTool={togglingTrackingToolId === tool.id}
                  skillsDirConfigured={Boolean(normalizedPath(draft.skillsDir))}
                  locale={locale}
                  t={t}
                  onDraftChange={(nextDraft) =>
                    setPathDrafts((prev) => ({ ...prev, [tool.id]: nextDraft }))
                  }
                  onPickToolPath={(target) => void handlePickToolPath(tool.id, target)}
                  onManualSync={() => void handleManualSync(tool)}
                  onToggleAutoSync={() => void handleToggleAutoSync(tool)}
                  onToggleTracking={() => void handleToggleTracking(tool)}
                  onSaveToolPaths={() => void handleSaveToolPaths(tool)}
                  onRemoveCustomTool={() => void handleRemoveCustomTool(tool.id)}
                />
              );
            })}
          </div>
        </section>
      </div>

      {showCustomForm && (
        <CustomToolFormCard
          form={form}
          submitting={submitting}
          t={t}
          onHide={() => setShowCustomForm(false)}
          onChange={setForm}
          onPickPath={(target) => void handlePickCustomFormPath(target)}
          onSubmit={() => void handleAddCustomTool()}
        />
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

