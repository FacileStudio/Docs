import { icons } from '@facile/muse';
import { describe, expect, it } from 'vitest';
import { documentIcon } from './brand';
import { SOLAR } from './icon-data';
import { NAMES, PICKABLE, pageIcon } from './icons';

const bundled = new Set<string>([...Object.values(icons), ...Object.keys(SOLAR), documentIcon]);

function drawn(name: string): boolean {
	return bundled.has(name);
}

describe('pageIcon', () => {
	it('keeps emoji as emoji', () => {
		expect(pageIcon('🚀')).toEqual({ emoji: '🚀' });
		expect(pageIcon('1️⃣')).toEqual({ emoji: '1️⃣' });
	});

	it('maps the Outline and Font Awesome names found in production to their own glyph', () => {
		expect(pageIcon('folder-closed')).toEqual({ glyph: 'solar:folder-linear' });
		expect(pageIcon('user-graduate')).toEqual({ glyph: 'solar:square-academic-cap-linear' });
		expect(pageIcon('academicCap')).toEqual({ glyph: 'solar:square-academic-cap-linear' });
		expect(pageIcon('book')).toEqual({ glyph: 'solar:book-2-linear' });
		expect(pageIcon('collection')).toEqual({ glyph: 'solar:folder-with-files-linear' });
	});

	it('draws the document icon for an empty or unknown name, never the name as text', () => {
		expect(pageIcon('')).toEqual({ glyph: documentIcon });
		expect(pageIcon(undefined)).toEqual({ glyph: documentIcon });
		expect(pageIcon('dragon-unknown')).toEqual({ glyph: documentIcon });
	});

	it('has bundled paths for every glyph it can return and every choice it offers', () => {
		for (const [glyph, names] of Object.entries(NAMES)) {
			expect(drawn(glyph), glyph).toBe(true);
			for (const name of names) expect(pageIcon(name), name).toEqual({ glyph });
		}
		for (const choice of PICKABLE) {
			const shown = pageIcon(choice.value);
			expect('glyph' in shown && shown.glyph !== documentIcon && drawn(shown.glyph)).toBe(true);
		}
	});
});
