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
