import type { EmbedPreview } from '@facile/encre';
import { backend } from './backend';
import type { AssistantProfile } from './types';

/** Where the Bulle chat embed lives. */
export const BULLE_ORIGIN = 'https://bulle.facile.studio';

/** The name shown until Bulle answers, or when it cannot. */
export const DEFAULT_ASSISTANT: AssistantProfile = { name: 'Bulle', accent: null };

/** What Docs tells Bulle about the open page. */
export interface PageContext {
	app: 'docs';
	id: string;
	title: string;
	url: string;
}

/** What the Bulle frame says to Docs. */
export type BulleMessage = { type: 'bulle:ready'; assistant: { name: string } } | { type: 'bulle:close' };

let asked: Promise<AssistantProfile> | null = null;

/** The person's assistant, asked once per tab; the default when it cannot be read, and asked again next time. */
export function assistant(): Promise<AssistantProfile> {
	asked ??= backend
		.assistant()
		.then((found) => (found.name.trim() ? found : DEFAULT_ASSISTANT))
		.catch(() => {
			asked = null;
			return DEFAULT_ASSISTANT;
		});
	return asked;
}

/** The chat embed URL for `page`. */
export function embedUrl(page: PageContext, origin = BULLE_ORIGIN): string {
	const query = new URLSearchParams({ app: page.app, id: page.id, title: page.title, url: page.url });
	return `${origin}/embed/chat?${query}`;
}

/** Whether `origin` may speak for Bulle: Bulle itself, or a localhost Bulle while developing. */
export function trustedOrigin(origin: string, dev = false): boolean {
	if (origin === BULLE_ORIGIN) return true;
	return dev && /^http:\/\/(localhost|127\.0\.0\.1)(:\d+)?$/.test(origin);
}

/** The Bulle message in `data`, or null when it is not one. */
export function bulleMessage(data: unknown): BulleMessage | null {
	if (typeof data !== 'object' || data === null) return null;
	const message = data as { type?: unknown; assistant?: { name?: unknown } };
	if (message.type === 'bulle:close') return { type: 'bulle:close' };
	if (message.type === 'bulle:ready' && typeof message.assistant?.name === 'string') {
		return { type: 'bulle:ready', assistant: { name: message.assistant.name } };
	}
	return null;
}

const OPEN_KEY = 'docs.bulle.open';

/** Whether `user` left the Bulle sidebar open; false when storage is unavailable. */
export function sidebarOpen(user: string): boolean {
	try {
		return localStorage.getItem(`${OPEN_KEY}.${user}`) === '1';
	} catch {
		return false;
	}
}

/** Remembers whether `user` has the Bulle sidebar open. */
export function rememberSidebar(user: string, open: boolean): void {
	try {
		localStorage.setItem(`${OPEN_KEY}.${user}`, open ? '1' : '0');
	} catch {
		return;
	}
}

/** The card preview of `url` from `GET /api/embeds/resolve`, without the fields encre does not show. */
export async function resolveEmbed(url: string): Promise<EmbedPreview> {
	const found = await backend.resolveEmbed(url);
	return {
		title: found.title,
		description: found.description ?? undefined,
		image: found.image ?? undefined,
		site: found.site ?? undefined
	};
}
