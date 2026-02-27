import { useI18n } from "../i18n/I18nProvider";
import { IconEdit } from "./icons";
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

  return (
    <article className="skill-card" onClick={onEdit}>
      <div className="skill-card-head">
        <h3 className="skill-card-name">{name}</h3>
        <button
          className="skill-card-edit"
          onClick={(e) => {
            e.stopPropagation();
            onEdit();
          }}
          title={t("skill.edit")}
        >
          <IconEdit size={14} />
        </button>
      </div>

      <p className="skill-card-desc">{description || t("skill.noDesc")}</p>

      <div className="skill-card-footer">
        {category && <span className="skill-card-category">{category}</span>}
        {tags && tags.length > 0 && (
          <div className="skill-card-tags">
            {tags.slice(0, 3).map((tag) => (
              <span key={tag} className="skill-card-tag">
                {tag}
              </span>
            ))}
            {tags.length > 3 && <span className="skill-card-tag">+{tags.length - 3}</span>}
          </div>
        )}
      </div>
    </article>
  );
}
