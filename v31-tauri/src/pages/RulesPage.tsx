import Editor from "@monaco-editor/react";
import { useEffect, useState } from "react";

import { rulesGet, rulesSave } from "../api/tauri";
import { IconSave } from "../components/icons";
import "./RulesPage.css";

export default function RulesPage() {
    const [content, setContent] = useState("");
    const [status, setStatus] = useState("");
    const [saving, setSaving] = useState(false);

    useEffect(() => {
        setStatus("loading...");
        void rulesGet()
            .then((r) => { setContent(r.content); setStatus(""); })
            .catch((e: unknown) => setStatus(String(e)));
    }, []);

    async function handleSave() {
        setSaving(true);
        setStatus("saving...");
        try {
            await rulesSave(content);
            setStatus("saved");
        } catch (e) {
            setStatus(String(e));
        } finally {
            setSaving(false);
        }
    }

    return (
        <div className="page animate-fadein">
            <header className="page-header">
                <h1 className="page-title">Global Rules</h1>
                <div className="dash-actions">
                    <span className="page-count">{status}</span>
                    <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
                        <IconSave size={14} />
                        {saving ? "Saving..." : "Save"}
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
