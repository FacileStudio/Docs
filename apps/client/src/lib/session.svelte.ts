import { backend } from './backend';
import type { Role, SessionUser, Space } from './types';

/** What the caller may do with a document. */
export type Access = 'view' | 'edit' | 'manage';

const SPACE_KEY = 'docs.space';

/** The access a scope role grants (plan 7.4): personal is the owner's, `member` edits, `admin` and `owner` manage. */
export function accessFor(role: Role | 'personal' | undefined): Access {
	if (role === 'personal' || role === 'admin' || role === 'owner') return 'manage';
	if (role === 'member') return 'edit';
	return 'view';
}

function remembered(): string | null {
	try {
		return localStorage.getItem(SPACE_KEY);
	} catch {
		return null;
	}
}

function remember(id: string | null): void {
	try {
		if (id) localStorage.setItem(SPACE_KEY, id);
		else localStorage.removeItem(SPACE_KEY);
	} catch {
		return;
	}
}

/** The signed-in person, their spaces, and the active scope (`null` is personal). */
class Session {
	user = $state<SessionUser | null>(null);
	spaces = $state<Space[]>([]);
	spaceId = $state<string | null>(null);
	ready = $state(false);

	space = $derived(this.spaces.find((space) => space.id === this.spaceId) ?? null);

	async load(): Promise<void> {
		try {
			this.user = await backend.session();
			if (this.user) this.spaces = await backend.spaces();
		} catch {
			this.user = null;
		}
		const saved = remembered();
		this.spaceId = this.spaces.some((space) => space.id === saved) ? saved : null;
		this.ready = true;
	}

	select(id: string | null): void {
		this.spaceId = id;
		remember(id);
	}

	accessTo(spaceId: string | null): Access {
		if (spaceId === null) return accessFor('personal');
		return accessFor(this.spaces.find((space) => space.id === spaceId)?.role);
	}

	async logout(): Promise<void> {
		try {
			await backend.logout();
		} finally {
			this.user = null;
		}
	}
}

/** The one session of the app. */
export const session = new Session();
