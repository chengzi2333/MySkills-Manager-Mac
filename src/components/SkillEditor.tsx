import Editor from "@monaco-editor/react";
import { useEffect, useState } from "react";

import {
  skillsGetContent,
  skillsSaveContent,
  type SkillMeta,
} from "../api/tauri";
import {
  fromEditableDocument,
  toEditableDocument,
  type EditableSkillDocument,
} from "../domain/skillDocument";
import { tagsFromInput, tagsToInput } from "../domain/tagInput";
import { useI18n } from "../i18n/I18nProvider";
import { IconClose, IconSave } from "./icons";
import "./SkillEditor.css";

type Props = {
  skill: SkillMeta;
  onClose: () => void;
  onSaved: () => void;
};

export default function SkillEditor({ skill, onClose, onSaved }: Props) {
  const { t } = useI18n();
  const [doc, setDoc] = useState<EditableSkillDocument | null>(null);
  const [status, setStatus] = useState("");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setDoc(null);
    setStatus(t("editor.loading"));
    void skillsGetContent(skill.name)
      .then((content) => {
        setDoc(toEditableDocument(content));
        setStatus("");
      })
      .catch((err: unknown) => setStatus(String(err)));
  }, [skill.name, t]);

  async function handleSave() {
    if (!doc) return;
    setSaving(true);
    setStatus(t("editor.saving"));
    try {
      const content = fromEditableDocument(doc);
      await skillsSaveContent(skill.name, content);
      setStatus(t("editor.saved"));
      onSaved();
    } catch (err) {
      setStatus(String(err));
    } finally {
      setSaving(false);
    }
  }

  const meta = doc?.frontmatter;
  const tags = meta ? tagsToInput(meta.tags) : "";

  return (
    <div className="drawer-overlay" onClick={onClose}>
      <aside className="drawer" onClick={(e) => e.stopPropagation()}>
        <header className="drawer-header">
          <h2 className="drawer-title">{skill.name}</h2>
          <div className="drawer-actions">
            <span className="drawer-status">{status}</span>
            <button className="btn btn-primary" onClick={handleSave} disabled={saving || !doc}>
              <IconSave size={14} />
              {saving ? t("editor.saving") : t("editor.save")}
            </button>
            <button className="btn btn-ghost" onClick={onClose} title={t("editor.close")}>
              <IconClose size={16} />
            </button>
          </div>
        </header>

        <div className="drawer-body">
          <div className="drawer-meta">
            <label className="field">
              <span className="field-label">{t("editor.category")}</span>
              <input
                className="field-input"
                value={meta?.category ?? ""}
                onChange={(e) =>
                  setDoc((prev) =>
                    prev
                      ? {
                          ...prev,
                          frontmatter: {
                            ...prev.frontmatter,
                            category: e.target.value,
                          },
                        }
                      : prev,
                  )
                }
              />
            </label>
            <label className="field">
              <span className="field-label">{t("editor.tags")}</span>
              <input
                className="field-input"
                value={tags}
                onChange={(e) =>
                  setDoc((prev) =>
                    prev
                      ? {
                          ...prev,
                          frontmatter: {
                            ...prev.frontmatter,
                            tags: tagsFromInput(e.target.value),
                          },
                        }
                      : prev,
                  )
                }
              />
            </label>
            <label className="field field-wide">
              <span className="field-label">{t("editor.notes")}</span>
              <textarea
                className="field-input"
                rows={2}
                value={meta?.my_notes ?? ""}
                onChange={(e) =>
                  setDoc((prev) =>
                    prev
                      ? {
                          ...prev,
                          frontmatter: {
                            ...prev.frontmatter,
                            my_notes: e.target.value,
                          },
                        }
                      : prev,
                  )
                }
              />
            </label>
          </div>

          <div className="drawer-editor">
            <Editor
              language="markdown"
              options={{
                automaticLayout: true,
                fontSize: 13,
                minimap: { enabled: false },
                scrollBeyondLastLine: false,
                wordWrap: "on",
              }}
              value={doc?.body ?? ""}
              onChange={(value) => setDoc((prev) => (prev ? { ...prev, body: value ?? "" } : prev))}
            />
          </div>
        </div>
      </aside>
    </div>
  );
}
