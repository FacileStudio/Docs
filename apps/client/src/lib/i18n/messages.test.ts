import { describe, expect, it } from 'vitest';
import { catalogs } from './messages';

const locales = ['fr', 'es', 'de'] as const;
const pitch = ['landing.slogan', 'landing.subtitle', 'landing.line', 'landing.title', 'landing.meta'];
const pitchBanned = [/\bmcp\b/i, /arbre de pages/i, /export pdf/i, /self-host/i, /auto-héberg/i, /votre serveur/i];
const banned = [/souverain/i, /sovereign/i, /comment(aire|ario|ar)?s?\b/i, /kommentar/i, /du studio/i, /—/];

function placeholders(message: unknown): string {
	const texts = typeof message === 'string' ? [message] : Object.values(message as Record<string, string>);
	const names = texts.flatMap((text) => [...text.matchAll(/\{(\w+)\}/g)].map((match) => match[1]));
	return [...new Set(names)].sort().join(',');
}

function texts(): [string, string][] {
	return Object.values(catalogs).flatMap((catalog) =>
		Object.entries(catalog as Record<string, unknown>).map(
			([key, text]) => [key, JSON.stringify(text)] as [string, string]
		)
	);
}

describe('landing catalogs', () => {
	for (const locale of locales) {
		it(`${locale} has every English key with the same placeholders`, () => {
			const problems: string[] = [];
			for (const [key, english] of Object.entries(catalogs.en)) {
				const message = (catalogs[locale] as Record<string, unknown>)[key];
				if (!message) problems.push(`${key}: empty`);
				else if (placeholders(message) !== placeholders(english)) problems.push(`${key}: placeholders differ`);
			}
			expect(problems).toEqual([]);
		});
	}

	it('keeps protocols and hosting out of the pitch', () => {
		const problems = texts()
			.filter(([key]) => pitch.includes(key))
			.flatMap(([key, text]) =>
				pitchBanned.filter((pattern) => pattern.test(text)).map((pattern) => `${key}: ${pattern}`)
			);
		expect(problems).toEqual([]);
	});

	it('writes no false claim, no comment feature and no em dash anywhere', () => {
		const problems = texts().flatMap(([key, text]) =>
			banned.filter((pattern) => pattern.test(text)).map((pattern) => `${key}: ${pattern}`)
		);
		expect(problems).toEqual([]);
	});
});
