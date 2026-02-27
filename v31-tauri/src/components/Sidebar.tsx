import {
    IconDashboard,
    IconGit,
    IconLogs,
    IconRules,
    IconSettings,
    IconSkills,
    IconTools,
} from "./icons";
import ThemeToggle from "./ThemeToggle";
import "./Sidebar.css";

export type ViewName = "skills" | "dashboard" | "logs" | "tools" | "rules" | "git";

const navItems: { view: ViewName; icon: React.ReactNode; label: string }[] = [
    { view: "skills", icon: <IconSkills size={20} />, label: "Skills" },
    { view: "dashboard", icon: <IconDashboard size={20} />, label: "Dashboard" },
    { view: "logs", icon: <IconLogs size={20} />, label: "Logs" },
    { view: "tools", icon: <IconTools size={20} />, label: "Tools" },
    { view: "rules", icon: <IconRules size={20} />, label: "Rules" },
    { view: "git", icon: <IconGit size={20} />, label: "Git" },
];

type Props = {
    active: ViewName;
    onChange: (view: ViewName) => void;
};

export default function Sidebar({ active, onChange }: Props) {
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
                <button className="sidebar-item" title="Settings">
                    <IconSettings size={20} />
                </button>
            </div>
        </nav>
    );
}
