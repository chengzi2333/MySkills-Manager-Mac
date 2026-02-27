import { useCallback, useEffect, useState } from "react";

import { gitCommit, gitPush, gitStatus, type GitStatus } from "../api/tauri";
import KpiCard from "../components/KpiCard";
import "./GitPage.css";

export default function GitPage() {
    const [state, setState] = useState<GitStatus | null>(null);
    const [status, setStatus] = useState("");
    const [commitMessage, setCommitMessage] = useState("");
    const [actionStatus, setActionStatus] = useState("");
    const [committing, setCommitting] = useState(false);
    const [pushing, setPushing] = useState(false);

    const refreshStatus = useCallback(async () => {
        setStatus("loading...");
        try {
            const r = await gitStatus();
            setState(r);
            setStatus("");
        } catch (e: unknown) {
            setStatus(String(e));
        }
    }, []);

    useEffect(() => {
        void refreshStatus();
    }, [refreshStatus]);

    async function handleCommit() {
        if (committing || pushing) return;
        setCommitting(true);
        setActionStatus("Committing...");
        void (async () => {
            try {
                const result = await gitCommit(commitMessage);
                setActionStatus(`Commit created: ${result.hash.slice(0, 8)}`);
                setCommitMessage("");
                await refreshStatus();
            } catch (e: unknown) {
                setActionStatus(String(e));
            } finally {
                setCommitting(false);
            }
        })();
    }

    async function handlePush() {
        if (committing || pushing) return;
        setPushing(true);
        setActionStatus("Pushing...");
        try {
            await gitPush();
            setActionStatus("Push success");
            await refreshStatus();
        } catch (e: unknown) {
            setActionStatus(String(e));
        } finally {
            setPushing(false);
        }
    }

    return (
        <div className="page animate-fadein">
            <header className="page-header">
                <h1 className="page-title">Git</h1>
                {status && <span className="page-count">{status}</span>}
            </header>

            <div className="kpi-row">
                <KpiCard label="Branch" value={state?.branch ?? "-"} />
                <KpiCard label="Changed" value={state?.changed.length ?? 0} />
                <KpiCard label="Staged" value={state?.staged.length ?? 0} />
                    <KpiCard label="Untracked" value={state?.not_added.length ?? 0} />
            </div>

            <article className="chart-card">
                <h3 className="chart-title">Actions</h3>
                <div className="git-actions">
                    <label className="field git-field">
                        <span className="field-label">Commit Message</span>
                        <input
                            className="field-input"
                            value={commitMessage}
                            placeholder="feat: describe your change"
                            onChange={(e) => setCommitMessage(e.target.value)}
                        />
                    </label>
                    <div className="git-action-buttons">
                        <button
                            className="btn btn-primary"
                            onClick={() => void handleCommit()}
                            disabled={committing || pushing || commitMessage.trim().length === 0}
                        >
                            {committing ? "Committing..." : "Commit"}
                        </button>
                        <button
                            className="btn btn-ghost"
                            onClick={() => void handlePush()}
                            disabled={committing || pushing}
                        >
                            {pushing ? "Pushing..." : "Push"}
                        </button>
                        <button
                            className="btn btn-ghost"
                            onClick={() => void refreshStatus()}
                            disabled={committing || pushing}
                        >
                            Refresh
                        </button>
                    </div>
                    {actionStatus && <p className="git-action-status">{actionStatus}</p>}
                </div>
            </article>

            <div className="chart-row">
                <article className="chart-card">
                    <h3 className="chart-title">Changed Files</h3>
                    {(state?.changed.length ?? 0) === 0
                        ? <p className="empty-state">No changed files.</p>
                        : <ul className="item-list">{state?.changed.map((f) => <li key={f}>{f}</li>)}</ul>}
                </article>
                <article className="chart-card">
                    <h3 className="chart-title">Staged Files</h3>
                    {(state?.staged.length ?? 0) === 0
                        ? <p className="empty-state">No staged files.</p>
                        : <ul className="item-list">{state?.staged.map((f) => <li key={f}>{f}</li>)}</ul>}
                </article>
            </div>

            <article className="chart-card">
                <h3 className="chart-title">Untracked Files</h3>
                {(state?.not_added.length ?? 0) === 0
                    ? <p className="empty-state">No untracked files.</p>
                    : <ul className="item-list">{state?.not_added.map((f) => <li key={f}>{f}</li>)}</ul>}
            </article>
        </div>
    );
}
