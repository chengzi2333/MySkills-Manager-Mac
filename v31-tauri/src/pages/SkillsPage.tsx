import { useMemo, useState } from "react";
import SkillCard from "../components/SkillCard";
import SkillEditor from "../components/SkillEditor";
import { IconSearch } from "../components/icons";
import type { SkillMeta } from "../api/tauri";
import "./SkillsPage.css";

type Props = {
    skills: SkillMeta[];
    onRefresh: () => void;
};

export default function SkillsPage({ skills, onRefresh }: Props) {
    const [search, setSearch] = useState("");
    const [category, setCategory] = useState("all");
    const [editing, setEditing] = useState<SkillMeta | null>(null);

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

    return (
        <div className="page animate-fadein">
            <header className="page-header">
                <h1 className="page-title">Skills</h1>
                <span className="page-count">{skills.length} skills</span>
            </header>

            <div className="skills-toolbar">
                <div className="search-box">
                    <IconSearch size={16} />
                    <input
                        className="search-input"
                        placeholder="Search skills..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                    />
                </div>
                <select
                    className="filter-select"
                    value={category}
                    onChange={(e) => setCategory(e.target.value)}
                >
                    {categories.map((c) => (
                        <option key={c} value={c}>
                            {c === "all" ? "All categories" : c}
                        </option>
                    ))}
                </select>
            </div>

            {visible.length === 0 ? (
                <p className="empty-state">No skills found.</p>
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
