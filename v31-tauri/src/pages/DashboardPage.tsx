import ReactECharts from "echarts-for-react";
import { useEffect, useState } from "react";

import { statsGet, type SkillMeta, type StatsResult } from "../api/tauri";
import KpiCard from "../components/KpiCard";
import "./DashboardPage.css";

type Props = { skills: SkillMeta[] };

export default function DashboardPage({ skills }: Props) {
    const [days, setDays] = useState(30);
    const [stats, setStats] = useState<StatsResult | null>(null);
    const [status, setStatus] = useState("");

    useEffect(() => {
        void (async () => {
            setStatus("loading...");
            try {
                const r = await statsGet(days);
                setStats(r);
                setStatus("");
            } catch (e: unknown) {
                setStatus(String(e));
            }
        })();
    }, [days]);

    const topSkills = stats?.by_skill.slice(0, 15) ?? [];
    const byTool = stats?.by_tool ?? [];
    const byDay = stats?.by_day ?? [];

    return (
        <div className="page animate-fadein">
            <header className="page-header">
                <h1 className="page-title">Dashboard</h1>
                <div className="dash-actions">
                    <select
                        className="filter-select"
                        value={days}
                        onChange={(e) => setDays(Number(e.target.value))}
                    >
                        <option value={7}>Last 7 days</option>
                        <option value={30}>Last 30 days</option>
                        <option value={90}>Last 90 days</option>
                    </select>
                    {status && <span className="page-count">{status}</span>}
                </div>
            </header>

            <div className="kpi-row">
                <KpiCard label="Total Invocations" value={stats?.total_invocations ?? 0} />
                <KpiCard label="Active Skills" value={stats?.by_skill.length ?? 0} />
                <KpiCard label="Total Skills" value={skills.length} />
                <KpiCard label="Unused Skills" value={stats?.unused_skills.length ?? 0} />
            </div>

            <div className="chart-row">
                <article className="chart-card">
                    <h3 className="chart-title">Top Skills</h3>
                    <ReactECharts
                        option={{
                            tooltip: { trigger: "axis" },
                            xAxis: { type: "value" },
                            yAxis: { type: "category", data: topSkills.map((i) => i.name) },
                            grid: { left: 130, right: 20, top: 20, bottom: 20 },
                            series: [{ type: "bar", data: topSkills.map((i) => i.count), barMaxWidth: 24 }],
                        }}
                        style={{ height: 320 }}
                    />
                </article>

                <article className="chart-card">
                    <h3 className="chart-title">By Tool</h3>
                    <ReactECharts
                        option={{
                            tooltip: { trigger: "item" },
                            series: [{
                                type: "pie",
                                radius: "62%",
                                data: byTool.map((i) => ({ name: i.name, value: i.count })),
                            }],
                        }}
                        style={{ height: 320 }}
                    />
                </article>
            </div>

            <article className="chart-card">
                <h3 className="chart-title">Daily Trend</h3>
                <ReactECharts
                    option={{
                        tooltip: { trigger: "axis" },
                        xAxis: { type: "category", data: byDay.map((i) => i.date) },
                        yAxis: { type: "value" },
                        grid: { left: 50, right: 20, top: 20, bottom: 40 },
                        series: [{ type: "line", data: byDay.map((i) => i.count), smooth: true }],
                    }}
                    style={{ height: 300 }}
                />
            </article>

            <article className="chart-card">
                <h3 className="chart-title">Unused Skills</h3>
                {(stats?.unused_skills.length ?? 0) === 0 ? (
                    <p className="empty-state">No unused skills in this time window.</p>
                ) : (
                    <ul className="item-list">
                        {stats?.unused_skills.map((n) => <li key={n}>{n}</li>)}
                    </ul>
                )}
            </article>
        </div>
    );
}
