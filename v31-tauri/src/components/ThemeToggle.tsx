import { IconMoon, IconSun } from "./icons";
import "./ThemeToggle.css";

export default function ThemeToggle() {
    function toggle() {
        const root = document.documentElement;
        const next = root.getAttribute("data-theme") === "dark" ? "light" : "dark";
        root.setAttribute("data-theme", next);
        localStorage.setItem("theme", next);
    }

    /* read initial preference */
    const current =
        typeof localStorage !== "undefined"
            ? localStorage.getItem("theme") ?? "light"
            : "light";
    if (typeof document !== "undefined") {
        document.documentElement.setAttribute("data-theme", current);
    }

    return (
        <button
            className="theme-toggle"
            onClick={toggle}
            title="Toggle theme"
            aria-label="Toggle theme"
        >
            {current === "dark" ? <IconSun size={18} /> : <IconMoon size={18} />}
        </button>
    );
}
