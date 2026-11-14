import { ApiError } from './backend';
import type { PendingAction } from './types';

/** How the confirm page describes an action's state: ready to run, or its page changed or vanished since. */
export const STATES: Record<string, { tone: 'success' | 'warning' | 'danger'; label: string }> = {
	ready: { tone: 'success', label: 'Prête' },
	changed: { tone: 'warning', label: 'La page a changé depuis la demande : la confirmation échouera' },
	missing: { tone: 'danger', label: 'La page n’existe plus ou vous n’y avez plus accès' }
};

/** The page an action touches, read from its target (`docs_replace` stores `{"document_id",…}`), or null. */
export function targetDocument(action: Pick<PendingAction, 'target'>): string | null {
	try {
		const target = JSON.parse(action.target) as { document_id?: unknown };
		return typeof target.document_id === 'string' ? target.document_id : null;
	} catch {
		return null;
	}
}

/** What the confirm page says when the API refuses the session: only a browser sign-in can confirm. */
export const BROWSER_ONLY = {
	title: 'Reconnectez-vous dans votre navigateur',
	description:
		'Seule une session ouverte dans votre navigateur peut confirmer une demande d’agent. L’application mobile ou de bureau ne le peut pas.'
};

/** Whether `error` is the confirm page's 403: the session is not the person's own browser sign-in. */
export function needsBrowserLogin(error: unknown): boolean {
	return error instanceof ApiError && error.status === 403 && error.code === 'permission_denied';
}

/** The message shown for any other failure. */
export function failureMessage(error: unknown, fallback: string): string {
	return error instanceof Error && error.message ? error.message : fallback;
}
