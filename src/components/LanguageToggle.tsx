import { useI18n } from "../i18n/I18nProvider";
import "./LanguageToggle.css";

export default function LanguageToggle() {
  const { locale, setLocale, t } = useI18n();
  const next = locale === "zh-CN" ? "en-US" : "zh-CN";
  const label = locale === "zh-CN" ? "EN" : "CN";

  return (
    <button
      className="language-toggle"
      onClick={() => setLocale(next)}
      title={t("locale.switch")}
      aria-label={t("locale.switch")}
    >
      {label}
    </button>
  );
}
