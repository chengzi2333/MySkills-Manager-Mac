import { useCallback, useEffect, useMemo, useState } from "react";

import {
  setupAddCustomTool,
  setupApply,
  setupRemoveCustomTool,
  setupStatus,
  type SetupApplyResult,
  type ToolStatus,
} from "../api/tauri";
import { useI18n } from "../i18n/I18nProvider";
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
  const [selected, setSelected] = useState<Record<string, boolean>>({});
  const [status, setStatus] = useState("");
  const [applyResults, setApplyResults] = useState<SetupApplyResult[]>([]);
  const [form, setForm] = useState<CustomToolForm>(EMPTY_FORM);
  const [busy, setBusy] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const loadStatus = useCallback(async () => {
    setBusy(true);
    setStatus(t("tools.loading"));
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
  }, [t]);

  useEffect(() => {
    void loadStatus();
  }, [loadStatus]);

  const selectedToolIds = useMemo(
    () => tools.filter((tool) => selected[tool.id]).map((tool) => tool.id),
    [tools, selected],
  );

  async function handleApply() {
    if (selectedToolIds.length === 0) {
      setStatus(t("tools.apply.none"));
      return;
    }
    setBusy(true);
    setStatus(t("tools.syncing"));
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
      setStatus("名称 / ID / 技能目录为必填项。");
      return;
    }

    setSubmitting(true);
    setStatus("正在添加自定义工具...");
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
      setStatus("已添加自定义工具");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleRemoveCustomTool(id: string) {
    setBusy(true);
    setStatus("正在移除自定义工具...");
    try {
      await setupRemoveCustomTool(id);
      await loadStatus();
      setStatus("已移除自定义工具");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="page animate-fadein">
      <header className="page-header">
        <h1 className="page-title">{t("tools.title")}</h1>
        <div className="tools-header-actions">
          <span className="page-count">{status || `${tools.length} ${t("tools.title")}`}</span>
          <button className="btn btn-ghost" onClick={() => void loadStatus()} disabled={busy}>
            {t("tools.refresh")}
          </button>
          <button
            className="btn btn-primary"
            onClick={() => void handleApply()}
            disabled={busy || selectedToolIds.length === 0}
          >
            {busy ? t("tools.syncing") : t("tools.apply.button", { count: selectedToolIds.length })}
          </button>
        </div>
      </header>

      <div className="tools-grid">
        {tools.map((tool) => (
          <article key={tool.id} className="tools-card">
            <header className="tools-card-header">
              <h3 className="tools-card-title">
                {tool.name}
                {tool.isCustom && <span className="tools-chip">{t("tools.custom")}</span>}
              </h3>
              <label className="tools-toggle">
                <input
                  type="checkbox"
                  checked={Boolean(selected[tool.id])}
                  onChange={(e) => setSelected((prev) => ({ ...prev, [tool.id]: e.target.checked }))}
                />
                <span>{t("tools.select")}</span>
              </label>
            </header>

            <dl className="tools-metrics">
              <div>
                <dt>{t("tools.syncedSkills")}</dt>
                <dd>{tool.syncedSkills}</dd>
              </div>
              <div>
                <dt>{t("tools.syncMode")}</dt>
                <dd>{tool.syncMode}</dd>
              </div>
            </dl>

            <div className="tools-flags">
              <span className={`tools-flag ${tool.exists ? "ok" : "warn"}`}>
                {tool.exists ? t("tools.flag.exists") : t("tools.flag.missing")}
              </span>
              <span className={`tools-flag ${tool.configured ? "ok" : "warn"}`}>
                {tool.configured ? t("tools.flag.rulesOk") : t("tools.flag.rulesMissing")}
              </span>
              {tool.id === "claude-code" && (
                <span className={`tools-flag ${tool.hookConfigured ? "ok" : "warn"}`}>
                  {tool.hookConfigured ? t("tools.flag.hookOk") : t("tools.flag.hookMissing")}
                </span>
              )}
            </div>

            <p className="tools-path">
              <strong>{t("tools.path.skills")}:</strong> {tool.skillsDir}
            </p>
            {tool.rulesPath && (
              <p className="tools-path">
                <strong>{t("tools.path.rules")}:</strong> {tool.rulesPath}
              </p>
            )}

            {tool.isCustom && (
              <button className="btn btn-ghost" onClick={() => void handleRemoveCustomTool(tool.id)} disabled={busy}>
                {t("tools.remove")}
              </button>
            )}
          </article>
        ))}
      </div>

      <article className="chart-card tools-form-card">
        <h3 className="chart-title">{t("tools.form.title")}</h3>
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
            <input
              className="field-input"
              value={form.skillsDir}
              onChange={(e) => setForm((prev) => ({ ...prev, skillsDir: e.target.value }))}
              placeholder="C:\\Users\\Keith\\.aider\\skills"
            />
          </label>
          <label className="field field-wide">
            <span className="field-label">{t("tools.form.rulesFile")}</span>
            <input
              className="field-input"
              value={form.rulesFile}
              onChange={(e) => setForm((prev) => ({ ...prev, rulesFile: e.target.value }))}
              placeholder="C:\\Users\\Keith\\.aider\\AGENTS.md"
            />
          </label>
          <label className="field">
            <span className="field-label">{t("tools.form.icon")}</span>
            <input
              className="field-input"
              value={form.icon}
              onChange={(e) => setForm((prev) => ({ ...prev, icon: e.target.value }))}
              placeholder="optional"
            />
          </label>
        </div>
        <div className="tools-form-actions">
          <button className="btn btn-primary" onClick={() => void handleAddCustomTool()} disabled={submitting}>
            {submitting ? t("tools.form.adding") : t("tools.form.add")}
          </button>
        </div>
      </article>

      {applyResults.length > 0 && (
        <article className="chart-card">
          <h3 className="chart-title">{t("tools.result.title")}</h3>
          <div className="tools-results">
            {applyResults.map((result) => (
              <div key={result.tool} className="tools-result-item">
                <strong>{result.tool}</strong>
                <span className={`tools-flag ${result.success ? "ok" : "warn"}`}>
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
