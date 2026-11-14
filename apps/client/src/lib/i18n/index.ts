import { browser } from '$app/environment';
import { createTranslator, LOCALE_COOKIE, resolveLocale, type Locale } from '@facile/muse';
import { catalogs } from './messages';

let active: () => Locale = () => 'fr';

function cookieLocale(): string | null {
	if (!browser) return null;
	const prefix = `${LOCALE_COOKIE}=`;
	const found = document.cookie.split('; ').find((part) => part.startsWith(prefix));
	return found ? decodeURIComponent(found.slice(prefix.length)) : null;
}

/** The visitor's locale for a signed-out page: the picked cookie, then the browser's language. */
export function visitorLocale(): string | null {
	return cookieLocale() ?? (browser ? navigator.language : null);
}

/** Persists the locale picked on a signed-out page for the next visit. */
export function rememberLocale(locale: Locale) {
	document.cookie = `${LOCALE_COOKIE}=${locale}; path=/; max-age=31536000; samesite=lax`;
}

/** Points the translator and the formats at the locale `provideLocale` resolved. */
export function bindLocale(source: () => Locale) {
	active = source;
}

/** The locale currently in force. */
export function locale(): Locale {
	return resolveLocale(active());
}

/** A whole amount in euros, written the way the current locale writes it. */
export function formatEuros(value: number): string {
	return new Intl.NumberFormat(locale(), { style: 'currency', currency: 'EUR', maximumFractionDigits: 0 }).format(
		value
	);
}

/** A short day and month, written the way the current locale writes it. */
export function formatDay(date: Date): string {
	return new Intl.DateTimeFormat(locale(), { day: 'numeric', month: 'short', timeZone: 'UTC' }).format(date);
}

/** Translates a landing message in the current locale. */
export const t = createTranslator(catalogs, () => active());
