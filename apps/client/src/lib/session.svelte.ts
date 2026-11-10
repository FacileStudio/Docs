import { backend } from './backend';
import type { Access, Role, SessionUser, Space } from './types';

const SPACE_KEY = 'docs.space';

/**
 * The access a scope role grants on its pages, as `docs/api.md` defines it: personal pages are their owner's,
 * `member` edits, `admin` and `owner` manage. A page answers its own `access`; this is for scope-wide actions only.
 */
export function scopeAccess(role: Role | 'personal' | undefined): Access {
	if (role === 'personal' || role === 'admin' || role === 'owner') return 'manager';
	if (role === 'member') return 'editor';
	return 'viewer';
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

	scopeAccess(spaceId: string | null): Access {
		if (spaceId === null) return scopeAccess('personal');
		return scopeAccess(this.spaces.find((space) => space.id === spaceId)?.role);
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
