import i18next from "i18next";
import { initReactI18next } from "react-i18next";
import { ApiError } from "../lib/api";
import { logger } from "../lib/logger";
import { en } from "./en";
import { fr } from "./fr";

type Widen<T> = { [K in keyof T]: T[K] extends string ? string : Widen<T[K]> };
export type Messages = Widen<typeof fr>;

export const LANGUAGES = ["fr", "en"] as const;
export type Language = (typeof LANGUAGES)[number];

const STORAGE_KEY = "cinelist.language";

function storedLanguage(): Language | null {
  try {
    const value = window.localStorage.getItem(STORAGE_KEY);
    return LANGUAGES.find((language) => language === value) ?? null;
  } catch {
    return null;
  }
}

function browserLanguage(): Language {
  return navigator.language.toLowerCase().startsWith("fr") ? "fr" : "en";
}

export function setLanguage(language: Language): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, language);
  } catch (error) {
    logger.warn("i18n.store_failed", { error: String(error) });
  }
  void i18next.changeLanguage(language);
  document.documentElement.lang = language;
}

export function initI18n(language: Language = storedLanguage() ?? browserLanguage()) {
  document.documentElement.lang = language;
  return i18next.use(initReactI18next).init({
    resources: { fr: { translation: fr }, en: { translation: en } },
    lng: language,
    fallbackLng: "fr",
    interpolation: { escapeValue: false },
  });
}

/** Translation key of the message explaining `error`. */
export function errorKey(error: unknown): string {
  if (error instanceof ApiError && error.code in fr.errors) {
    return `errors.${error.code}`;
  }
  return "errors.generic";
}
