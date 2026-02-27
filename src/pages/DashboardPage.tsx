import ReactECharts from "echarts-for-react";
import { useEffect, useState } from "react";
import * as echarts from "echarts";

import { statsGet, type SkillMeta, type StatsResult } from "../api/tauri";
import KpiCard from "../components/KpiCard";
import { useI18n } from "../i18n/I18nProvider";
import "./DashboardPage.css";

type Props = { skills: SkillMeta[] };

const LIGHT_THEME_NAME = "myskills-light";
const DARK_THEME_NAME = "myskills-dark";
let themesRegistered = false;

function ensureEchartsThemes() {
  if (themesRegistered) return;
  echarts.registerTheme(LIGHT_THEME_NAME, {
    backgroundColor: "transparent",
    color: ["#2f6bff", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6", "#06b6d4"],
    textStyle: { color: "#0f172a" },
    title: { textStyle: { color: "#0f172a" } },
    legend: { textStyle: { color: "#334155" } },
    tooltip: {
      backgroundColor: "rgba(15, 23, 42, 0.92)",
      borderColor: "#1e293b",
      textStyle: { color: "#e2e8f0" },
    },
    categoryAxis: {
      axisLine: { lineStyle: { color: "#cbd5e1" } },
      axisTick: { lineStyle: { color: "#cbd5e1" } },
      axisLabel: { color: "#475569" },
      splitLine: { lineStyle: { color: "#e2e8f0" } },
    },
    valueAxis: {
      axisLine: { lineStyle: { color: "#cbd5e1" } },
      axisTick: { lineStyle: { color: "#cbd5e1" } },
      axisLabel: { color: "#475569" },
      splitLine: { lineStyle: { color: "#e2e8f0" } },
    },
  });

  echarts.registerTheme(DARK_THEME_NAME, {
    backgroundColor: "transparent",
    color: ["#4f8cff", "#34d399", "#fbbf24", "#f87171", "#a78bfa", "#22d3ee"],
    textStyle: { color: "#e8eaed" },
    title: { textStyle: { color: "#e8eaed" } },
    legend: { textStyle: { color: "#cbd5e1" } },
    tooltip: {
      backgroundColor: "rgba(15, 23, 42, 0.94)",
      borderColor: "#334155",
      textStyle: { color: "#e2e8f0" },
    },
    categoryAxis: {
      axisLine: { lineStyle: { color: "#334155" } },
      axisTick: { lineStyle: { color: "#334155" } },
      axisLabel: { color: "#cbd5e1" },
      splitLine: { lineStyle: { color: "#1e293b" } },
    },
    valueAxis: {
      axisLine: { lineStyle: { color: "#334155" } },
      axisTick: { lineStyle: { color: "#334155" } },
      axisLabel: { color: "#cbd5e1" },
      splitLine: { lineStyle: { color: "#1e293b" } },
    },
  });

  themesRegistered = true;
}

function getThemeName() {
  return document.documentElement.getAttribute("data-theme") === "dark" ? DARK_THEME_NAME : LIGHT_THEME_NAME;
}

export default function DashboardPage({ skills }: Props) {
  const { t } = useI18n();
  ensureEchartsThemes();
  const [days, setDays] = useState(30);
  const [stats, setStats] = useState<StatsResult | null>(null);
  const [status, setStatus] = useState("");
  const [themeName, setThemeName] = useState(getThemeName());

  useEffect(() => {
    void (async () => {
      setStatus(t("tools.loading"));
      try {
        const r = await statsGet(days);
        setStats(r);
        setStatus("");
      } catch (e: unknown) {
        setStatus(String(e));
      }
    })();
  }, [days, t]);

  useEffect(() => {
    const observer = new MutationObserver(() => setThemeName(getThemeName()));
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    return () => observer.disconnect();
  }, []);

  const topSkills = stats?.by_skill.slice(0, 15) ?? [];
  const byTool = stats?.by_tool ?? [];
  const byDay = stats?.by_day ?? [];

  return (
    <div className="page animate-fadein">
      <header className="page-header">
        <h1 className="page-title">{t("dashboard.title")}</h1>
        <div className="dash-actions">
          <select className="filter-select" value={days} onChange={(e) => setDays(Number(e.target.value))}>
            <option value={7}>{t("dashboard.range.7")}</option>
            <option value={30}>{t("dashboard.range.30")}</option>
            <option value={90}>{t("dashboard.range.90")}</option>
          </select>
          {status && <span className="page-count">{status}</span>}
        </div>
      </header>

      <div className="kpi-row">
        <KpiCard label={t("dashboard.kpi.invocations")} value={stats?.total_invocations ?? 0} />
        <KpiCard label={t("dashboard.kpi.active")} value={stats?.by_skill.length ?? 0} />
        <KpiCard label={t("dashboard.kpi.total")} value={skills.length} />
        <KpiCard label={t("dashboard.kpi.unused")} value={stats?.unused_skills.length ?? 0} />
      </div>

      <div className="chart-row">
        <article className="chart-card">
          <h3 className="chart-title">{t("dashboard.topSkills")}</h3>
          <ReactECharts
            theme={themeName}
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
          <h3 className="chart-title">{t("dashboard.byTool")}</h3>
          <ReactECharts
            theme={themeName}
            option={{
              tooltip: { trigger: "item" },
              series: [
                {
                  type: "pie",
                  radius: "62%",
                  data: byTool.map((i) => ({ name: i.name, value: i.count })),
                },
              ],
            }}
            style={{ height: 320 }}
          />
        </article>
      </div>

      <article className="chart-card">
        <h3 className="chart-title">{t("dashboard.byDay")}</h3>
        <ReactECharts
          theme={themeName}
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
        <h3 className="chart-title">{t("dashboard.unused")}</h3>
        {(stats?.unused_skills.length ?? 0) === 0 ? (
          <p className="empty-state">{t("dashboard.unused.empty")}</p>
        ) : (
          <ul className="item-list">{stats?.unused_skills.map((n) => <li key={n}>{n}</li>)}</ul>
        )}
      </article>
    </div>
  );
}
