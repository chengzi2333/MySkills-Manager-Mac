import { useEffect, useState } from "react";

import { appPing, skillsList, type SkillMeta } from "./api/tauri";
import Sidebar, { type ViewName } from "./components/Sidebar";
import DashboardPage from "./pages/DashboardPage";
import GitPage from "./pages/GitPage";
import LogsPage from "./pages/LogsPage";
import RulesPage from "./pages/RulesPage";
import SkillsPage from "./pages/SkillsPage";
import ToolsPage from "./pages/ToolsPage";
import "./App.css";

export default function App() {
  const [view, setView] = useState<ViewName>("skills");
  const [skills, setSkills] = useState<SkillMeta[]>([]);
  const [ping, setPing] = useState("checking...");

  function loadSkills() {
    void skillsList()
      .then(setSkills)
      .catch((err: unknown) => console.error("skills_list error:", err));
  }

  useEffect(() => {
    void appPing().then(setPing).catch(() => setPing("error"));
    loadSkills();
  }, []);

  function renderPage() {
    switch (view) {
      case "skills":
        return <SkillsPage skills={skills} onRefresh={loadSkills} />;
      case "dashboard":
        return <DashboardPage skills={skills} />;
      case "logs":
        return <LogsPage skills={skills} />;
      case "tools":
        return <ToolsPage />;
      case "rules":
        return <RulesPage />;
      case "git":
        return <GitPage />;
    }
  }

  return (
    <div className="app-shell">
      <Sidebar active={view} onChange={setView} />
      <div className="app-content">
        <div className="app-status-bar">
          <span className="status-dot" />
          <span className="status-text">app_ping: {ping}</span>
        </div>
        {renderPage()}
      </div>
    </div>
  );
}
