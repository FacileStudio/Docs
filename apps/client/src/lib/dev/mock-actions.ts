import type { PendingAction } from '../types.ts';
import { NO_CONTENT, fail, json, type Route } from './mock-share.ts';

/** The confirm page routes of `/api/mcp/actions`, over the mock's pending actions. */
export function actionRoutes(get: () => { actions: PendingAction[] }): Route[] {
	const take = (id: string) => {
		const state = get();
		const found = state.actions.some((a) => a.id === id);
		state.actions = state.actions.filter((a) => a.id !== id);
		return found;
	};
	const missing = () => fail(404, 'not_found', 'this action was already confirmed, dismissed or expired');
	return [
		['GET', /^\/mcp\/actions$/, () => json(get().actions)],
		['POST', /^\/mcp\/actions\/(?<id>[^/]+)\/confirm$/, (p) => (take(p.id) ? json({ status: 'done' }) : missing())],
		[
			'POST',
			/^\/mcp\/actions\/(?<id>[^/]+)\/dismiss$/,
			(p) => (take(p.id) ? new Response(null, NO_CONTENT) : missing())
		]
	];
}
