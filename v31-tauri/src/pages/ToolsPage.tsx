import "./ToolsPage.css";

export default function ToolsPage() {
    return (
        <div className="page animate-fadein">
            <header className="page-header">
                <h1 className="page-title">Tools</h1>
                <span className="page-count">Coming soon — F7 Setup Wizard</span>
            </header>

            <div className="tools-placeholder">
                <p>
                    This page will display detected AI tools (Claude Code, Codex,
                    Antigravity, etc.) as cards with enable/disable toggles and
                    configuration status once the <code>setup_status</code> and{" "}
                    <code>setup_apply</code> backend commands are implemented.
                </p>
            </div>
        </div>
    );
}
