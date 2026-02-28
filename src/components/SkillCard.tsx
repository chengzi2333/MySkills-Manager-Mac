import { useMemo } from "react";

import { useI18n } from "../i18n/I18nProvider";
import "./SkillCard.css";

type Props = {
  name: string;
  description?: string;
  category?: string;
  tags?: string[];
  onEdit: () => void;
};

export default function SkillCard({
  name,
  description,
  category,
  tags,
  onEdit,
}: Props) {
  const { t } = useI18n();

  const chips = useMemo(() => {
    const out: string[] = [];
    if (category) {
      out.push(category);
    }
    for (const tag of tags ?? []) {
      if (!out.includes(tag)) {
        out.push(tag);
      }
    }
    return out.slice(0, 3);
  }, [category, tags]);

  const avatarVariant = useMemo(() => {
    let hash = 0;
    for (let i = 0; i < name.length; i += 1) {
      hash = (hash << 5) - hash + name.charCodeAt(i);
      hash |= 0;
    }
    return Math.abs(hash) % 5;
  }, [name]);

  return (
    <article className="skill-card" onClick={onEdit}>
      <div className="skill-card-main">
        <div className={`skill-card-avatar skill-card-avatar-${avatarVariant}`} aria-hidden="true">
          <span className="skill-card-avatar-star">*</span>
        </div>
        <div className="skill-card-copy">
          <h3 className="skill-card-name">{name}</h3>
          <p className="skill-card-desc">{description || t("skill.noDesc")}</p>
        </div>
      </div>

      <div className="skill-card-divider" />

      <div className="skill-card-footer">
        <div className="skill-card-enable">
          <span className="skill-card-enable-label">{t("skills.enableFor")}</span>
          {chips.length > 0 ? (
            <div className="skill-card-tags">
              {chips.map((chip) => (
                <span key={chip} className="skill-card-tag">
                  {chip}
                </span>
              ))}
              {(tags?.length ?? 0) > chips.length && (
                <span className="skill-card-tag">+{(tags?.length ?? 0) - chips.length}</span>
              )}
            </div>
          ) : (
            <span className="skill-card-empty">{t("skills.enableFor.empty")}</span>
          )}
        </div>
        <button
          className="skill-card-edit"
          onClick={(e) => {
            e.stopPropagation();
            onEdit();
          }}
          title={t("skill.edit")}
        >
          {t("skill.edit")}
        </button>
      </div>
    </article>
  );
}
