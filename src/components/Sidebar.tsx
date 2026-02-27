import {
    IconDashboard,
    IconGit,
    IconLogs,
    IconRules,
    IconSettings,
    IconSkills,
    IconTools,
} from "./icons";
import { useI18n } from "../i18n/I18nProvider";
import LanguageToggle from "./LanguageToggle";
import ThemeToggle from "./ThemeToggle";
import "./Sidebar.css";

export type ViewName = "skills" | "dashboard" | "logs" | "tools" | "rules" | "git";

type Props = {
    active: ViewName;
    onChange: (view: ViewName) => void;
};

export default function Sidebar({ active, onChange }: Props) {
    const { t } = useI18n();
    const navItems: { view: ViewName; icon: React.ReactNode; label: string }[] = [
        { view: "skills", icon: <IconSkills size={20} />, label: t("nav.skills") },
        { view: "dashboard", icon: <IconDashboard size={20} />, label: t("nav.dashboard") },
        { view: "logs", icon: <IconLogs size={20} />, label: t("nav.logs") },
        { view: "tools", icon: <IconTools size={20} />, label: t("nav.tools") },
        { view: "rules", icon: <IconRules size={20} />, label: t("nav.rules") },
        { view: "git", icon: <IconGit size={20} />, label: "Git" },
    ];

    return (
        <nav className="sidebar-nav">
            <div className="sidebar-top">
                <div className="sidebar-logo">M</div>
                {navItems.map((item) => (
                    <button
                        key={item.view}
                        className={`sidebar-item${active === item.view ? " active" : ""}`}
                        onClick={() => onChange(item.view)}
                        title={item.label}
                    >
                        {item.icon}
                        <span className="sidebar-label">{item.label}</span>
                    </button>
                ))}
            </div>

            <div className="sidebar-bottom">
                <ThemeToggle />
                <LanguageToggle />
                <button className="sidebar-item" title={t("nav.settings")}>
                    <IconSettings size={20} />
                </button>
            </div>
        </nav>
    );
}
