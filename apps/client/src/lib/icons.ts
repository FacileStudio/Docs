import { icons as museIcons, registerIcons } from '@facile/muse';
import { documentIcon } from './brand';
import { SOLAR } from './icon-data';

const line = (body: string) => ({
	body: `<g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">${body}</g>`,
	width: 24,
	height: 24
});

registerIcons(SOLAR);
registerIcons({
	'docs:cut': line(
		'<circle cx="6" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><path d="M8.2 8.2L20 20M8.2 15.8L20 4"/>'
	),
	'docs:paste': line(
		'<rect x="8" y="2" width="8" height="4" rx="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>'
	),
	'docs:link': line(
		'<path d="M10 14a4 4 0 0 0 5.66 0l3-3a4 4 0 0 0-5.66-5.66l-1 1"/><path d="M14 10a4 4 0 0 0-5.66 0l-3 3a4 4 0 0 0 5.66 5.66l1-1"/>'
	),
	'docs:table': line('<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 10h18M3 15h18M9 4v16"/>'),
	'docs:columns': line('<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M12 4v16"/>'),
	'docs:badge': line('<path d="M3 12V5a2 2 0 0 1 2-2h7l9 9-9 9z"/><circle cx="8" cy="8" r="1.5"/>'),
	'docs:h1': line('<path d="M4 6v12M12 6v12M4 12h8M17 10l3-2v10"/>'),
	'docs:h2': line('<path d="M4 6v12M12 6v12M4 12h8M16 10a2.5 2.5 0 0 1 5 0c0 2-5 4-5 8h5"/>'),
	'docs:h3': line('<path d="M4 6v12M12 6v12M4 12h8M16 8h5l-3 4a3 3 0 1 1-2 5"/>'),
	'docs:bullets': line(
		'<path d="M9 6h11M9 12h11M9 18h11"/><circle cx="4.5" cy="6" r="1"/><circle cx="4.5" cy="12" r="1"/><circle cx="4.5" cy="18" r="1"/>'
	),
	'docs:numbers': line(
		'<path d="M10 6h10M10 12h10M10 18h10M4 4.5l1.5-1V9M3.5 14.5a1.5 1.5 0 0 1 3 0c0 1.2-3 2.3-3 4h3"/>'
	),
	'docs:tasks': line(
		'<rect x="3" y="4" width="6" height="6" rx="1.5"/><path d="M4.5 7l1 1 2-2M13 7h8M13 17h8"/><rect x="3" y="14" width="6" height="6" rx="1.5"/>'
	),
	'docs:quote': line('<path d="M5 18c2-1 3-3 3-6V7H4v5h4M15 18c2-1 3-3 3-6V7h-4v5h4"/>'),
	'docs:rule': line('<path d="M3 12h18M7 7h10M7 17h10"/>'),
	'docs:diagram': line(
		'<rect x="3" y="3" width="7" height="5" rx="1.5"/><rect x="14" y="16" width="7" height="5" rx="1.5"/><path d="M6.5 8v5.5a2.5 2.5 0 0 0 2.5 2.5h5"/>'
	)
});

const BLOCKS: Record<string, string> = {
	paragraph: museIcons.text,
	heading1: 'docs:h1',
	heading2: 'docs:h2',
	heading3: 'docs:h3',
	bulletList: 'docs:bullets',
	orderedList: 'docs:numbers',
	taskList: 'docs:tasks',
	blockquote: 'docs:quote',
	codeBlock: museIcons.code,
	horizontalRule: 'docs:rule',
	table: 'docs:table',
	callout: museIcons.info,
	stat: 'solar:hashtag-linear',
	chart: museIcons.dashboard,
	diagram: 'docs:diagram',
	columns: 'docs:columns',
	badge: 'docs:badge'
};

/** The glyph of an encre block command, by its id; plain text for one Docs does not know yet. */
export function blockIcon(id: string): string {
	return BLOCKS[id] ?? museIcons.text;
}

export const icons = {
	...museIcons,
	more: 'solar:menu-dots-linear',
	cut: 'docs:cut',
	paste: 'docs:paste',
	link: 'docs:link',
	table: 'docs:table',
	columns: 'docs:columns',
	badge: 'docs:badge'
} as const;

/** How a page icon draws: an emoji as text, anything else as a muse glyph. */
export type PageIcon = { emoji: string } | { glyph: string };

/** The icons the chooser offers next to the emoji, as stored values and their French labels. */
export const PICKABLE: readonly { value: string; label: string }[] = [
	{ value: 'folder', label: 'Dossier' },
	{ value: 'collection', label: 'Collection' },
	{ value: 'book', label: 'Livre' },
	{ value: 'academic-cap', label: 'Formation' },
	{ value: 'team', label: 'Équipe' },
	{ value: 'code', label: 'Code' },
	{ value: 'idea', label: 'Idée' },
	{ value: 'rocket', label: 'Lancement' },
	{ value: 'calendar', label: 'Agenda' },
	{ value: 'globe', label: 'Web' },
	{ value: 'padlock', label: 'Sécurité' },
	{ value: 'database', label: 'Données' },
	{ value: 'star', label: 'Favori' },
	{ value: 'flag', label: 'Objectif' },
	{ value: 'chat', label: 'Discussion' }
];

const folder = icons.folder;
const book = 'solar:book-2-linear';
const cap = 'solar:square-academic-cap-linear';
const collection = 'solar:folder-with-files-linear';
const team = icons.usersGroup;
const user = icons.userCircle;
const lock = 'solar:lock-keyhole-linear';
const image = 'solar:gallery-linear';
const chat = 'solar:chat-round-linear';
const chart = icons.dashboard;
const database = 'solar:database-linear';
const flask = 'solar:test-tube-linear';
const page = documentIcon;

/** Every glyph a page icon can draw, with the stored names (Docs, Outline, Font Awesome) that draw it. */
export const NAMES: Record<string, string[]> = {
	[folder]: ['folder', 'folder-closed', 'folder-open', 'folder-tree', 'archive'],
	[collection]: ['collection', 'folders', 'layer-group', 'building-blocks', 'shapes'],
	[book]: ['book', 'book-open', 'manual', 'book-bookmark', 'book-open-reader'],
	[cap]: ['academic-cap', 'user-graduate', 'graduation-cap', 'school'],
	['solar:library-linear']: ['library', 'book-atlas', 'landmark'],
	['solar:notebook-linear']: ['notebook', 'note-sticky', 'clipboard', 'clipboard-list', 'list-check'],
	[team]: ['team', 'users', 'people-group', 'user-group', 'handshake'],
	[user]: ['user', 'user-tie', 'circle-user', 'id-card', 'address-card'],
	[icons.code]: ['code', 'terminal', 'laptop-code', 'file-code', 'brackets-curly'],
	[icons.ideas]: ['idea', 'light', 'light-bulb', 'lightbulb'],
	['solar:rocket-linear']: ['rocket', 'promote', 'plane', 'paper-plane'],
	[icons.calendar]: ['calendar', 'calendar-days', 'calendar-check'],
	[icons.clock]: ['clock', 'hourglass', 'stopwatch'],
	[icons.globe]: ['globe', 'internet', 'earth-europe', 'earth-americas', 'globe-europe'],
	[icons.monitor]: ['browser', 'desktop', 'display', 'laptop', 'tv'],
	[lock]: ['padlock', 'lock', 'unlock'],
	[icons.key]: ['key'],
	[icons.shield]: ['shield', 'shield-halved', 'shield-check'],
	[database]: ['database', 'hard-drive'],
	[icons.server]: ['server', 'cloud', 'network-wired'],
	['solar:star-linear']: ['star', 'star-half'],
	['solar:flag-linear']: ['flag', 'bullseye', 'crosshairs'],
	['solar:heart-linear']: ['heart', 'thumbs-up', 'thumbsup'],
	[chat]: ['chat', 'comment', 'comments', 'feedback', 'message', 'messages'],
	[icons.mail]: ['email', 'envelope', 'inbox', 'at'],
	['solar:camera-linear']: ['camera', 'video', 'film'],
	[image]: ['image', 'images', 'photo-film'],
	[icons.palette]: ['palette', 'paintbrush', 'brush', 'pen-ruler'],
	[icons.edit]: ['pencil', 'pen', 'pen-to-square', 'signature'],
	[chart]: ['graph', 'chart-line', 'chart-simple', 'chart-pie', 'chart-column', 'gauge'],
	['solar:fire-linear']: ['flame', 'fire'],
	['solar:leaf-linear']: ['leaf', 'seedling', 'tree', 'carrot'],
	['solar:bookmark-linear']: ['bookmark'],
	['solar:hashtag-linear']: ['hashtag', 'hash'],
	[flask]: ['beaker', 'flask', 'vial', 'microscope'],
	['solar:case-linear']: ['briefcase', 'suitcase', 'business-time'],
	['solar:map-linear']: ['map', 'location-dot', 'map-location-dot', 'route'],
	['solar:cart-large-linear']: ['cart-shopping', 'basket-shopping', 'store', 'shop'],
	['solar:music-note-linear']: ['music', 'headphones'],
	['solar:bug-linear']: ['bug'],
	['solar:box-linear']: ['box', 'boxes-stacked', 'truck', 'vehicle'],
	['solar:smile-circle-linear']: ['smiley', 'face-smile'],
	[icons.bolt]: ['lightning', 'bolt'],
	[icons.sun]: ['sun'],
	[icons.moon]: ['moon'],
	[icons.warning]: ['warning', 'triangle-exclamation', 'circle-exclamation'],
	[icons.help]: ['question', 'circle-question', 'circle-info', 'info'],
	[icons.settings]: ['tools', 'gear', 'gears', 'wrench', 'screwdriver-wrench'],
	[icons.home]: ['house', 'home'],
	[icons.card]: ['coins', 'money', 'credit-card', 'wallet', 'euro-sign'],
	[icons.eye]: ['eye', 'glasses'],
	[page]: ['file', 'file-lines', 'note', 'document']
};

function normal(name: string): string {
	return name.toLowerCase().replace(/[-_\s]/g, '');
}

const GLYPHS = new Map(Object.entries(NAMES).flatMap(([glyph, names]) => names.map((name) => [normal(name), glyph])));

/**
 * The icon a page's stored `icon` draws. Emoji stay emoji. A name (Docs' own picks, or Outline's
 * `academicCap` and Font Awesome `folder-closed`) draws its muse glyph, and a name with no glyph
 * draws the document icon: a stored name never shows up as text.
 */
export function pageIcon(value: string | null | undefined): PageIcon {
	const icon = value?.trim() ?? '';
	if (!icon) return { glyph: page };
	if (!/[a-z]/i.test(icon)) return { emoji: icon };
	return { glyph: GLYPHS.get(normal(icon)) ?? page };
}
