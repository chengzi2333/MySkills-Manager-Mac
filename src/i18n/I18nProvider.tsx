import { createContext, useContext, useMemo, useState, type ReactNode } from "react";

import { messages, type Locale, type MessageKey } from "./messages";

const STORAGE_KEY = "myskills.locale";

type I18nContextValue = {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: MessageKey, params?: Record<string, string | number>) => string;
};

const I18nContext = createContext<I18nContextValue | null>(null);

function readInitialLocale(): Locale {
  if (typeof window === "undefined") return "zh-CN";
  const raw = window.localStorage.getItem(STORAGE_KEY);
  return raw === "en-US" ? "en-US" : "zh-CN";
}

function interpolate(template: string, params?: Record<string, string | number>) {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (_, key: string) => String(params[key] ?? `{${key}}`));
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(readInitialLocale);

  const value = useMemo<I18nContextValue>(
    () => ({
      locale,
      setLocale(next) {
        setLocaleState(next);
        if (typeof window !== "undefined") {
          window.localStorage.setItem(STORAGE_KEY, next);
        }
      },
      t(key, params) {
        const template = messages[locale][key] ?? messages["zh-CN"][key];
        return interpolate(template, params);
      },
    }),
    [locale],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

// eslint-disable-next-line react-refresh/only-export-components
export function useI18n() {
  const value = useContext(I18nContext);
  if (!value) {
    throw new Error("useI18n must be used within I18nProvider");
  }
  return value;
}
