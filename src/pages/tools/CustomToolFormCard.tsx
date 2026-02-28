import { IconClose, IconFolder } from "../../components/icons";
import type { MessageKey } from "../../i18n/messages";
import type { PathPickerTarget } from "../toolsPathPicker";
import type { CustomToolForm } from "./customToolForm";

type TranslateFn = (
  key: MessageKey,
  params?: Record<string, string | number>,
) => string;

type CustomToolFormCardProps = {
  form: CustomToolForm;
  submitting: boolean;
  t: TranslateFn;
  onHide: () => void;
  onChange: (next: CustomToolForm) => void;
  onPickPath: (target: PathPickerTarget) => void;
  onSubmit: () => void;
};

export default function CustomToolFormCard({
  form,
  submitting,
  t,
  onHide,
  onChange,
  onPickPath,
  onSubmit,
}: CustomToolFormCardProps) {
  return (
    <article className="chart-card tools-form-card">
      <header className="tools-form-head">
        <h3 className="chart-title">{t("tools.form.title")}</h3>
        <button className="btn btn-ghost tools-form-hide-btn" onClick={onHide}>
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
            onChange={(e) => onChange({ ...form, name: e.target.value })}
            placeholder="Aider"
          />
        </label>
        <label className="field">
          <span className="field-label">{t("tools.form.id")}</span>
          <input
            className="field-input"
            value={form.id}
            onChange={(e) => onChange({ ...form, id: e.target.value })}
            placeholder="aider"
          />
        </label>
        <label className="field field-wide">
          <span className="field-label">{t("tools.form.skillsDir")}</span>
          <div className="tool-path-row">
            <input
              className="field-input tools-path-input"
              value={form.skillsDir}
              onChange={(e) => onChange({ ...form, skillsDir: e.target.value })}
              placeholder="C:\\Users\\Keith\\.aider\\skills"
            />
            <button
              type="button"
              className="tool-path-picker"
              onClick={() => onPickPath("skills")}
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
              onChange={(e) => onChange({ ...form, rulesFile: e.target.value })}
              placeholder="C:\\Users\\Keith\\.aider\\AGENTS.md"
            />
            <button
              type="button"
              className="tool-path-picker"
              onClick={() => onPickPath("rules")}
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
            onChange={(e) => onChange({ ...form, icon: e.target.value })}
            placeholder="openai | anthropic | /tool-logos/custom.svg"
          />
        </label>
      </div>
      <div className="tools-form-actions">
        <button className="btn btn-primary" onClick={onSubmit} disabled={submitting}>
          {submitting ? t("tools.form.adding") : t("tools.form.add")}
        </button>
      </div>
    </article>
  );
}
