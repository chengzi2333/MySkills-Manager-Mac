import { useMemo, useState } from "react";

import {
  onboardingImportInstalledSkills,
  setupLocalSkillsOverview,
  type LocalSkillsOverview,
  type SkillMeta,
} from "../api/tauri";
import { useI18n } from "../i18n/I18nProvider";
import SkillCard from "../components/SkillCard";
import SkillEditor from "../components/SkillEditor";
import { IconRefresh, IconSearch } from "../components/icons";
import "./SkillsPage.css";

type Props = {
  skills: SkillMeta[];
  onRefresh: () => void;
};

export default function SkillsPage({ skills, onRefresh }: Props) {
  const { t } = useI18n();
  const [search, setSearch] = useState("");
  const [category, setCategory] = useState("all");
  const [editing, setEditing] = useState<SkillMeta | null>(null);
  const [overviewBusy, setOverviewBusy] = useState(false);
  const [syncMissingBusy, setSyncMissingBusy] = useState(false);
  const [overviewStatus, setOverviewStatus] = useState("");
  const [overview, setOverview] = useState<LocalSkillsOverview | null>(null);

  const categories = useMemo(() => {
    const out = new Set<string>();
    for (const s of skills) {
      if (s.category) out.add(s.category);
    }
    return ["all", ...Array.from(out).sort()];
  }, [skills]);

  const visible = useMemo(() => {
    const q = search.trim().toLowerCase();
    return skills.filter((s) => {
      if (category !== "all" && (s.category ?? "") !== category) return false;
      if (!q) return true;
      const tags = (s.tags ?? []).join(" ").toLowerCase();
      const notes = (s.my_notes ?? "").toLowerCase();
      return (
        s.name.toLowerCase().includes(q) ||
        (s.description ?? "").toLowerCase().includes(q) ||
        tags.includes(q) ||
        notes.includes(q)
      );
    });
  }, [category, search, skills]);

  async function handleLocalOverview() {
    setOverviewBusy(true);
    setOverviewStatus("正在扫描本机工具技能...");
    try {
      const result = await setupLocalSkillsOverview();
      setOverview(result);
      if (result.tools.length === 0) {
        setOverviewStatus("未识别到可导入的工具技能。");
      } else {
        const summary = `共识别 ${result.totalSkills} 个技能（唯一 ${result.uniqueSkills}）；已同步 ${result.matchedInMySkills}，未收录 ${result.missingInMySkills}，冲突 ${result.conflictWithMySkills}。`;
        setOverviewStatus(
          result.missingInMySkills > 0
            ? `${summary} 可点击下方按钮同步未收录技能。`
            : summary,
        );
        if (result.missingInMySkills > 0) {
          return;
        }
        if (result.conflictWithMySkills > 0) {
          setOverviewStatus(`${summary} 存在同名但内容不同的技能，请手动处理冲突。`);
        } else {
          setOverviewStatus(`${summary} 当前无需同步。`);
        }
      }
    } catch (e: unknown) {
      setOverviewStatus(String(e));
    } finally {
      setOverviewBusy(false);
    }
  }

  async function handleSyncMissingSkills() {
    if (!overview || overview.missingInMySkills === 0) return;

    setSyncMissingBusy(true);
    setOverviewStatus(`开始同步 ${overview.missingInMySkills} 个未收录技能...`);
    try {
      const syncResult = await onboardingImportInstalledSkills();
      const refreshed = await setupLocalSkillsOverview();
      setOverview(refreshed);
      setOverviewStatus(
        `已同步 ${syncResult.importedTotal}/${syncResult.detectedTotal}，跳过已存在 ${syncResult.skippedExistingTotal}。当前未收录 ${refreshed.missingInMySkills}。`,
      );
      onRefresh();
    } catch (e: unknown) {
      setOverviewStatus(String(e));
    } finally {
      setSyncMissingBusy(false);
    }
  }

  return (
    <div className="page animate-fadein skills-page">
      <header className="page-header skills-page-header">
        <div className="skills-header-copy">
          <h1 className="page-title">{t("skills.title")}</h1>
          <p className="skills-installed">{t("skills.installed", { count: skills.length })}</p>
        </div>
        <div className="skills-header-actions">
          <button
            className="btn btn-primary skills-overview-btn"
            onClick={() => void handleLocalOverview()}
            disabled={overviewBusy}
          >
            {overviewBusy ? "扫描中..." : "全览本机 skills"}
          </button>
          <button className="btn btn-ghost skills-refresh-btn" onClick={onRefresh}>
            <IconRefresh size={14} />
            {t("skills.refresh")}
          </button>
          <div className="search-box skills-search-box">
            <IconSearch size={16} />
            <input
              className="search-input"
              placeholder={t("skills.search")}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
          <select
            className="filter-select skills-filter-select"
            value={category}
            onChange={(e) => setCategory(e.target.value)}
          >
            {categories.map((c) => (
              <option key={c} value={c}>
                {c === "all" ? t("skills.category.all") : c}
              </option>
            ))}
          </select>
        </div>
      </header>

      {(overviewStatus || overview) && (
        <section className="skills-overview-panel">
          {overviewStatus && <p className="skills-overview-status">{overviewStatus}</p>}
          {overview && (
            <>
              <p className="skills-overview-summary">
                工具数 {overview.tools.length}，技能总数 {overview.totalSkills}，唯一技能 {overview.uniqueSkills}
                ，已同步 {overview.matchedInMySkills}，未收录 {overview.missingInMySkills}，冲突{" "}
                {overview.conflictWithMySkills}
              </p>
              <div className="skills-overview-legend">
                <span className="skills-overview-tag matched">哈希一致</span>
                <span className="skills-overview-tag missing">未收录</span>
                <span className="skills-overview-tag conflict">内容冲突</span>
              </div>
              {overview.missingInMySkills > 0 && (
                <div className="skills-overview-actions">
                  <button
                    className="btn btn-primary"
                    onClick={() => void handleSyncMissingSkills()}
                    disabled={syncMissingBusy}
                  >
                    {syncMissingBusy ? "同步中..." : `同步未收录 skills (${overview.missingInMySkills})`}
                  </button>
                </div>
              )}
              {overview.duplicateNames.length > 0 && (
                <div className="skills-overview-duplicates">
                  <strong>重名技能：</strong>
                  <div className="skills-overview-tags">
                    {overview.duplicateNames.map((name) => (
                      <span key={name} className="skills-overview-tag duplicate">
                        {name}
                      </span>
                    ))}
                  </div>
                </div>
              )}
              <div className="skills-overview-tools">
                {overview.tools.map((tool) => (
                  <article key={tool.toolId} className="skills-overview-tool-card">
                    <h3>
                      {tool.toolName} ({tool.count})
                    </h3>
                    <div className="skills-overview-tags">
                      {tool.skills.map((skill) => (
                        <span
                          key={`${tool.toolId}-${skill.name}`}
                          className={`skills-overview-tag ${
                            skill.hashConflictsMySkills
                              ? "conflict"
                              : skill.hashMatchesMySkills
                                ? "matched"
                                : skill.duplicateAcrossTools
                                  ? "duplicate"
                                  : skill.inMySkills
                                    ? "tracked"
                                    : "missing"
                          }`}
                        >
                          {skill.name}
                        </span>
                      ))}
                    </div>
                  </article>
                ))}
              </div>
            </>
          )}
        </section>
      )}

      {visible.length === 0 ? (
        <p className="empty-state">{t("skills.empty")}</p>
      ) : (
        <div className="skills-grid">
          {visible.map((skill) => (
            <SkillCard
              key={skill.name}
              name={skill.name}
              description={skill.description}
              category={skill.category}
              tags={skill.tags}
              onEdit={() => setEditing(skill)}
            />
          ))}
        </div>
      )}

      {editing && (
        <SkillEditor
          skill={editing}
          onClose={() => setEditing(null)}
          onSaved={() => {
            onRefresh();
            setEditing(null);
          }}
        />
      )}
    </div>
  );
}
