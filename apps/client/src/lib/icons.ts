import { icons, registerIcons } from '@facile/muse';
import { documentIcon } from './brand';
import { SOLAR } from './icon-data';

registerIcons(SOLAR);

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
