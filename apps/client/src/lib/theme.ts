import { applyTheme, isTheme, type Theme } from '@facile/muse';

const KEY = 'docs.theme';

/** Applies the theme this browser saved, if any. */
export function restoreTheme(): void {
	try {
		const saved = localStorage.getItem(KEY);
		if (isTheme(saved)) applyTheme(saved);
	} catch {
		return;
	}
}

/** Applies `theme` and saves it for this browser. */
export function saveTheme(theme: Theme): void {
	applyTheme(theme);
	try {
		localStorage.setItem(KEY, theme);
	} catch {
		return;
	}
}
