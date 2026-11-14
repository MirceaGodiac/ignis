// Theme: follow the system, or pin light or dark. The choice is a per-viewer convenience kept in
// localStorage; the page renders correctly without it (system). `<html data-theme>` carries a pinned
// choice, and styles.css reads it before `prefers-color-scheme`.
import { useEffect, useState } from "react";

export type ThemeChoice = "system" | "light" | "dark";
export type Scheme = "light" | "dark";

const KEY = "ignis.theme";
const EVENT = "ignis:theme";

export function storedTheme(): ThemeChoice {
  try {
    const v = localStorage.getItem(KEY);
    return v === "light" || v === "dark" ? v : "system";
  } catch {
    return "system";
  }
}

const systemDark = () => window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;

export function currentScheme(): Scheme {
  const t = document.documentElement.dataset.theme;
  if (t === "light" || t === "dark") return t;
  return systemDark() ? "dark" : "light";
}

export function applyTheme(choice: ThemeChoice) {
  const root = document.documentElement;
  if (choice === "system") delete root.dataset.theme;
  else root.dataset.theme = choice;
  try {
    if (choice === "system") localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, choice);
  } catch {
    /* storage blocked: the choice lasts for this page only */
  }
  window.dispatchEvent(new Event(EVENT));
}

/** The scheme in effect, updated on a system change or a pinned choice. Charts rebuild on it. */
export function useScheme(): Scheme {
  const [scheme, setScheme] = useState<Scheme>(currentScheme);
  useEffect(() => {
    const on = () => setScheme(currentScheme());
    const mq = window.matchMedia?.("(prefers-color-scheme: dark)");
    mq?.addEventListener("change", on);
    window.addEventListener(EVENT, on);
    return () => {
      mq?.removeEventListener("change", on);
      window.removeEventListener(EVENT, on);
    };
  }, []);
  return scheme;
}

export function useThemeChoice(): [ThemeChoice, (c: ThemeChoice) => void] {
  const [choice, setChoice] = useState<ThemeChoice>(storedTheme);
  return [
    choice,
    (c) => {
      setChoice(c);
      applyTheme(c);
    },
  ];
}
