import { useMemo, useState } from "react";

type Props = {
  id: string;
  name: string;
  icon?: string;
};

const OFFICIAL_ICON_ASSETS: Record<string, string> = {
  openai: "/tool-logos/openai-symbol.svg",
  anthropic: "/tool-logos/anthropic.svg",
  gemini: "/tool-logos/googlegemini.svg",
  googlegemini: "/tool-logos/googlegemini.svg",
  cursor: "/tool-logos/cursor.svg",
  windsurf: "/tool-logos/windsurf.svg",
  android: "/tool-logos/android.svg",
  antigravity: "/tool-logos/antigravity.svg",
  trae: "/tool-logos/trae.svg",
  augment: "/tool-logos/augment.svg",
  openclaw: "/tool-logos/openclaw.svg",
  opencode: "/tool-logos/opencode.svg",
};

const TOOL_ICON_BY_ID: Record<string, string> = {
  codex: OFFICIAL_ICON_ASSETS.openai,
  "claude-code": OFFICIAL_ICON_ASSETS.anthropic,
  cursor: OFFICIAL_ICON_ASSETS.cursor,
  windsurf: OFFICIAL_ICON_ASSETS.windsurf,
  droid: OFFICIAL_ICON_ASSETS.android,
  gemini: OFFICIAL_ICON_ASSETS.gemini,
  "gemini-cli": OFFICIAL_ICON_ASSETS.gemini,
  antigravity: OFFICIAL_ICON_ASSETS.antigravity,
  trae: OFFICIAL_ICON_ASSETS.trae,
  augment: OFFICIAL_ICON_ASSETS.augment,
  openclaw: OFFICIAL_ICON_ASSETS.openclaw,
  opencode: OFFICIAL_ICON_ASSETS.opencode,
};

function normalizeKey(value: string) {
  return value.trim().toLowerCase();
}

function isImageSource(value: string) {
  if (!value) {
    return false;
  }
  return (
    value.startsWith("/") ||
    value.startsWith("asset:") ||
    value.startsWith("tauri:") ||
    value.startsWith("data:image/") ||
    /^https?:\/\//i.test(value)
  );
}

export default function ToolLogo({ id, name, icon }: Props) {
  const [loadError, setLoadError] = useState(false);
  const iconValue = icon?.trim() ?? "";

  const imageSource = useMemo(() => {
    if (isImageSource(iconValue)) {
      return iconValue;
    }

    const iconKey = normalizeKey(iconValue);
    if (iconKey && OFFICIAL_ICON_ASSETS[iconKey]) {
      return OFFICIAL_ICON_ASSETS[iconKey];
    }

    const toolKey = normalizeKey(id);
    if (TOOL_ICON_BY_ID[toolKey]) {
      return TOOL_ICON_BY_ID[toolKey];
    }

    return "";
  }, [iconValue, id]);

  if (imageSource && !loadError) {
    return (
      <div className="tool-card-logo is-brand" aria-hidden="true">
        <img
          src={imageSource}
          alt=""
          className="tool-card-logo-img"
          onError={() => setLoadError(true)}
        />
      </div>
    );
  }

  return (
    <div className="tool-card-logo is-fallback" aria-hidden="true">
      <span className="tool-card-logo-fallback">{name.slice(0, 1).toUpperCase()}</span>
    </div>
  );
}
