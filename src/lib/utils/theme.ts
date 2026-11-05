import type { Theme } from "@/bindings";

/**
 * Appearance handling for candyAI.
 *
 * candyAI is dark only. There is no light palette and no OS following, so
 * this module exists to keep the upstream call sites working while pinning
 * the document to the dark palette.
 */

export const THEME_STORAGE_KEY = "candyai.theme";

/** Kept for the settings type; only "dark" is meaningful. */
export const THEME_OPTIONS: Theme[] = ["dark"];

/** Pin the document to dark. Any requested theme is ignored. */
export const applyTheme = (_theme?: Theme): void => {
  const root = document.documentElement;
  root.dataset.theme = "dark";
  try {
    localStorage.setItem(THEME_STORAGE_KEY, "dark");
  } catch {
    // localStorage may be unavailable; the palette is still forced by CSS.
  }
};

/** Always dark. */
export const getStoredTheme = (): Theme => "dark";

/** Pin dark again after settings load. */
export const syncThemeFromSettings = async (): Promise<void> => {
  applyTheme("dark");
};
