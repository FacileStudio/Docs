import { registerIcons } from '@facile/muse';

/** The brand mark of Docs, the one bold-duotone glyph in the rail. */
export const logoIcon = 'docs:logo';

/** A document row in the tree, for a document without its own icon. */
export const documentIcon = 'docs:document';

/** Opens the share dialog. */
export const shareIcon = 'docs:share';

const sheet = 'M4 6c0-2.8 0-4 1.2-5S8.2 0 11 0h2l7 7v11c0 2.8 0 4-1.2 5s-3 1-5.8 1h-2c-2.8 0-4.6 0-5.8-1S4 20.8 4 18Z';
const fold = 'M13 0v4c0 1.9 0 2.8.6 3.4S15.1 8 17 8h3.9L13 0ZM8 13h8v1.5H8Zm0 4h5v1.5H8Z';
const top = 'M14 2.5H9.5c-2.8 0-4.2 0-5.1.9S3.5 5.7 3.5 8.5v7c0 2.8 0 4.2.9 5.1s2.3.9 5.1.9h5';
const page = `${top}c2.8 0 4.2 0 5.1-.9s.9-2.3.9-5.1V8.5Z`;
const lines = 'M14 2.5v3c0 1.4 0 2.1.4 2.6s1.2.4 2.6.4h3.5M8 13h8M8 17h5';
const stroke = 'fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"';
const nodes = '<circle cx="18" cy="5" r="2.5"/><circle cx="6" cy="12" r="2.5"/><circle cx="18" cy="19" r="2.5"/>';

registerIcons({
	[logoIcon]: {
		body: `<path fill="currentColor" opacity=".5" d="${sheet}"/><path fill="currentColor" d="${fold}"/>`,
		width: 24,
		height: 24
	},
	[documentIcon]: {
		body: `<g ${stroke}><path d="${page}"/><path d="${lines}"/></g>`,
		width: 24,
		height: 24
	},
	[shareIcon]: {
		body: `<g ${stroke}>${nodes}<path d="m8.2 10.8 7.6-4.6M8.2 13.2l7.6 4.6"/></g>`,
		width: 24,
		height: 24
	}
});
