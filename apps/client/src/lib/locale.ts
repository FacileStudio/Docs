import type { Locale } from '@facile/encre';

const SUPPORTED: readonly Locale[] = ['fr', 'en', 'es', 'de'];

/** The first of `languages` (BCP 47 tags, preferred first) that encre speaks, else French like the rest of Docs. */
export function pickLocale(languages: readonly string[]): Locale {
	for (const tag of languages) {
		const base = tag.toLowerCase().split('-')[0] as Locale;
		if (SUPPORTED.includes(base)) return base;
	}
	return 'fr';
}

/** The encre locale of this browser. */
export function browserLocale(): Locale {
	if (typeof navigator === 'undefined') return 'fr';
	return pickLocale(navigator.languages?.length ? navigator.languages : [navigator.language ?? '']);
}

/** The copy Docs hands the editor itself, in each encre locale. */
export const EDITOR_COPY: Record<Locale, { placeholder: string; uploadFailed: string }> = {
	fr: { placeholder: 'Écrivez, ou tapez / pour insérer un bloc', uploadFailed: 'L’image n’a pas pu être envoyée' },
	en: { placeholder: 'Write, or type / to insert a block', uploadFailed: 'The image could not be uploaded' },
	es: { placeholder: 'Escribe, o teclea / para insertar un bloque', uploadFailed: 'No se pudo subir la imagen' },
	de: {
		placeholder: 'Schreiben Sie, oder tippen Sie /, um einen Block einzufügen',
		uploadFailed: 'Das Bild konnte nicht hochgeladen werden'
	}
};
