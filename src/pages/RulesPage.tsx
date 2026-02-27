import Editor from "@monaco-editor/react";
import { useEffect, useState } from "react";

import { rulesGet, rulesSave } from "../api/tauri";
import { IconSave } from "../components/icons";
import { useI18n } from "../i18n/I18nProvider";
import "./RulesPage.css";

export default function RulesPage() {
  const { t } = useI18n();
  const [content, setContent] = useState("");
  const [status, setStatus] = useState("");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setStatus(t("tools.loading"));
    void rulesGet()
      .then((result) => {
        setContent(result.content);
        setStatus("");
      })
      .catch((e: unknown) => setStatus(String(e)));
  }, [t]);

  async function handleSave() {
    setSaving(true);
    setStatus(t("editor.saving"));
    try {
      await rulesSave(content);
      setStatus(t("editor.saved"));
    } catch (e) {
      setStatus(String(e));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="page animate-fadein">
      <header className="page-header">
        <h1 className="page-title">{t("rules.title")}</h1>
        <div className="dash-actions">
          <span className="page-count">{status}</span>
          <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
            <IconSave size={14} />
            {saving ? t("editor.saving") : t("editor.save")}
          </button>
        </div>
      </header>

      <div className="rules-editor-wrap">
        <Editor
          language="markdown"
          options={{
            automaticLayout: true,
            fontSize: 13,
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            wordWrap: "on",
          }}
          value={content}
          onChange={(v) => setContent(v ?? "")}
        />
      </div>
    </div>
  );
}
