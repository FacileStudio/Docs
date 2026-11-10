import type { DocumentRecord, TreeNode } from '../types.ts';

/** One mock route handler: the named groups of its pattern, the URL and the request. */
export type Handler = (params: Record<string, string>, url: URL, request: Request) => Promise<Response> | Response;

/** One mock route: its method, its path pattern and its handler. */
export type Route = [string, RegExp, Handler];

/** A share link as the mock stores it, token in the clear. */
export interface MockShare {
	id: string;
	token: string;
	document_id: string;
	children: boolean;
	revoked: boolean;
	created_at: string;
}

/** A JSON response. */
export const json = (body: unknown, status = 200) =>
	new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});

/** The suite error envelope. */
export const fail = (status: number, code: string, message: string) => json({ error: { code, message } }, status);

/** A request's JSON body, or `{}` when it has none. */
export const body = async (request: Request) =>
	request.headers.get('Content-Type')?.includes('json') ? request.json() : {};

/** The tree entry of a document. */
export function row(doc: DocumentRecord): TreeNode {
	const { id, parent_id, position, title, icon, updated_at, trashed_at } = doc;
	return { id, parent_id, position, title, icon, updated_at, trashed_at };
}

/** The init of a 204 answer. */
export const NO_CONTENT = { status: 204 };

/** The share link and public page routes, over the mock's state. */
export function shareRoutes(
	get: () => { shares: MockShare[]; documents: DocumentRecord[] },
	next: () => string,
	find: (id: string) => DocumentRecord | undefined
): Route[] {
	const live = (id: string) => get().shares.filter((s) => s.document_id === id && !s.revoked);
	return [
		[
			'POST',
			/^\/documents\/(?<id>[^/]+)\/share$/,
			async (params, _url, request) => {
				if (!find(params.id)) return fail(404, 'not_found', 'document not found');
				const id = next();
				const children = Boolean((await body(request)).children);
				const share = {
					id,
					token: `tok-${id}`,
					document_id: params.id,
					children,
					revoked: false,
					created_at: ''
				};
				get().shares.push(share);
				return json({ id, url: `/s/${share.token}` }, 201);
			}
		],
		[
			'GET',
			/^\/documents\/(?<id>[^/]+)\/share$/,
			(params) =>
				json(
					live(params.id).map(({ id, children, created_at }) => ({
						id,
						children,
						created_at
					}))
				)
		],
		[
			'DELETE',
			/^\/share\/(?<id>[^/]+)$/,
			(params) => {
				const share = get().shares.find((s) => s.id === params.id);
				if (!share) return fail(404, 'not_found', 'share not found');
				share.revoked = true;
				return new Response(null, NO_CONTENT);
			}
		],
		[
			'GET',
			/^\/public\/(?<token>[^/]+)$/,
			(params) => {
				const share = get().shares.find((s) => s.token === params.token && !s.revoked);
				const doc = share && find(share.document_id);
				if (!share || !doc) return fail(404, 'not_found', 'share not found');
				const children = share.children
					? get()
							.documents.filter((d) => d.parent_id === doc.id)
							.map(row)
					: [];
				return json({
					id: doc.id,
					title: doc.title,
					icon: doc.icon,
					content: doc.content,
					children
				});
			}
		]
	];
}
