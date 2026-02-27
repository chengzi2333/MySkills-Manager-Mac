import { useEffect, useState } from "react";

import { gitStatus, type GitStatus } from "../api/tauri";
import KpiCard from "../components/KpiCard";
import "./GitPage.css";

export default function GitPage() {
    const [state, setState] = useState<GitStatus | null>(null);
    const [status, setStatus] = useState("");

    useEffect(() => {
        void (async () => {
            setStatus("loading...");
            try {
                const r = await gitStatus();
                setState(r);
                setStatus("");
            } catch (e: unknown) {
                setStatus(String(e));
            }
        })();
    }, []);

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
