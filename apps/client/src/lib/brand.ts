import { registerIcons } from '@facile/muse';

/** The brand mark of Docs, the Solar documents glyph in the rail, matching the suite catalog. */
export const logoIcon = 'solar:documents-bold-duotone';

/** A document row in the tree, for a document without its own icon. */
export const documentIcon = 'docs:document';

/** Opens the share dialog. */
export const shareIcon = 'docs:share';

const top = 'M14 2.5H9.5c-2.8 0-4.2 0-5.1.9S3.5 5.7 3.5 8.5v7c0 2.8 0 4.2.9 5.1s2.3.9 5.1.9h5';
const page = `${top}c2.8 0 4.2 0 5.1-.9s.9-2.3.9-5.1V8.5Z`;
const lines = 'M14 2.5v3c0 1.4 0 2.1.4 2.6s1.2.4 2.6.4h3.5M8 13h8M8 17h5';
const stroke = 'fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"';
const nodes = '<circle cx="18" cy="5" r="2.5"/><circle cx="6" cy="12" r="2.5"/><circle cx="18" cy="19" r="2.5"/>';

registerIcons({
	[logoIcon]: {
		body: '<path fill="currentColor" fill-rule="evenodd" d="M5.87868 2.87868C5 3.75736 5 5.17157 5 8V16C5 18.8284 5 20.2426 5.87868 21.1213C6.75736 22 8.17157 22 11 22H13C15.8284 22 17.2426 22 18.1213 21.1213C19 20.2426 19 18.8284 19 16V8C19 5.17157 19 3.75736 18.1213 2.87868C17.2426 2 15.8284 2 13 2H11C8.17157 2 6.75736 2 5.87868 2.87868ZM8.25 17C8.25 16.5858 8.58579 16.25 9 16.25H12C12.4142 16.25 12.75 16.5858 12.75 17C12.75 17.4142 12.4142 17.75 12 17.75H9C8.58579 17.75 8.25 17.4142 8.25 17ZM9 12.25C8.58579 12.25 8.25 12.5858 8.25 13C8.25 13.4142 8.58579 13.75 9 13.75H15C15.4142 13.75 15.75 13.4142 15.75 13C15.75 12.5858 15.4142 12.25 15 12.25H9ZM8.25 9C8.25 8.58579 8.58579 8.25 9 8.25H15C15.4142 8.25 15.75 8.58579 15.75 9C15.75 9.41421 15.4142 9.75 15 9.75H9C8.58579 9.75 8.25 9.41421 8.25 9Z" clip-rule="evenodd"/><g fill="currentColor" opacity=".5"><path d="M5.23535 4.05811C5.00012 4.94134 5 6.17674 5 7.99951V15.9995C5 17.8225 5.00008 19.0586 5.23535 19.9419L5 19.9233C4.02513 19.8274 3.36875 19.6113 2.87891 19.1216C2.00023 18.2429 2 16.8279 2 13.9995V9.99951C2 7.17135 2.00027 5.75706 2.87891 4.87842C3.36877 4.38867 4.02505 4.17158 5 4.07568L5.23535 4.05811Z"/><path d="M19 4.07568C19.975 4.17159 20.6312 4.38865 21.1211 4.87842C21.9997 5.75706 22 7.17135 22 9.99951V13.9995C22 16.8279 21.9998 18.2429 21.1211 19.1216C20.6313 19.6113 19.9749 19.8274 19 19.9233L18.7646 19.9419C18.9999 19.0586 19 17.8225 19 15.9995V7.99951C19 6.17674 18.9999 4.94134 18.7646 4.05811L19 4.07568Z"/></g>',
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
