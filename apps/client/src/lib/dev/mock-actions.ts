import type { PendingAction } from '../types.ts';
import { fail, json, type Route } from './mock-share.ts';

/** The confirm page routes of `/api/mcp/pending`, over the mock's pending actions. */
export function actionRoutes(get: () => { actions: PendingAction[] }): Route[] {
	const take = (id: string) => {
		const state = get();
		const found = state.actions.some((a) => a.id === id);
		state.actions = state.actions.filter((a) => a.id !== id);
		return found;
	};
	const missing = () => fail(404, 'not_found', 'this action was already confirmed, dismissed or expired');
	return [
		['GET', /^\/mcp\/pending$/, () => json({ actions: get().actions })],
		['POST', /^\/mcp\/pending\/(?<id>[^/]+)\/confirm$/, (p) => (take(p.id) ? json({ status: 'done' }) : missing())],
		[
			'POST',
			/^\/mcp\/pending\/(?<id>[^/]+)\/dismiss$/,
			(p) => (take(p.id) ? json({ status: 'dismissed' }) : missing())
		]
	];
}
