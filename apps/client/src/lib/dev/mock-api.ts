import type { DocumentRecord, PendingAction, Revision, Space } from '../types.ts';
import { actionRoutes } from './mock-actions.ts';
import { body, fail, json, row, shareRoutes, type Handler, type MockShare } from './mock-share.ts';

type Params = Record<string, string>;

/** The in-memory state behind the mock API, exposed so tests can seed and inspect it. */
export interface MockState {
	user: { email: string; name: string } | null;
	spaces: Space[];
	documents: DocumentRecord[];
	revisions: Record<string, Revision[]>;
	shares: MockShare[];
	actions: PendingAction[];
	calls: { method: string; path: string; csrf: boolean; body: unknown }[];
}

const EMPTY = { type: 'doc', content: [{ type: 'paragraph' }] };

/** A mock of the `docs/api.md` HTTP API for local UI work (`DOCS_MOCK=1 bun run dev`) and for tests. Never shipped. */
export class MockApi {
	state: MockState;
	private counter = 0;

	constructor(seed: Partial<MockState> = {}) {
		this.state = {
			user: { email: 'camille@facile.studio', name: 'Camille' },
			spaces: [],
			documents: [],
			revisions: {},
			shares: [],
			actions: [],
			calls: [],
			...seed
		};
	}

	private next(): string {
		this.counter += 1;
		return `mock-${this.counter}`;
	}

	private now(): string {
		return new Date(Date.now() + this.counter).toISOString();
	}

	private find(id: string): DocumentRecord | undefined {
		return this.state.documents.find((doc) => doc.id === id);
	}

	private siblings(parent: string | null, except: string): DocumentRecord[] {
		return this.state.documents
			.filter((d) => d.parent_id === parent && d.id !== except && !d.trashed_at)
			.sort((a, b) => a.position - b.position);
	}

	private withDoc(
		handler: (doc: DocumentRecord, params: Params, request: Request) => Response | Promise<Response>
	): Handler {
		return (params, _url, request) => {
			const doc = this.find(params.id);
			return doc ? handler(doc, params, request) : fail(404, 'not_found', 'document not found');
		};
	}

	private routes: [string, RegExp, Handler][] = [
		['GET', /^\/auth\/session$/, () => json({ user: this.state.user })],
		['POST', /^\/auth\/logout$/, () => ((this.state.user = null), json({ logged_out: true }))],
		['GET', /^\/spaces$/, () => json({ spaces: this.state.spaces })],
		[
			'GET',
			/^\/documents$/,
			(_p, url) => {
				const space = url.searchParams.get('space_id');
				const trashed = url.searchParams.get('trashed') === 'true';
				const rows = this.state.documents.filter((d) => d.space_id === space && Boolean(d.trashed_at) === trashed);
				return json(rows.map(row));
			}
		],
		[
			'POST',
			/^\/documents$/,
			async (_p, _u, request) => {
				const input = await body(request);
				const stamp = this.now();
				const doc: DocumentRecord = {
					id: this.next(),
					space_id: input.space_id ?? null,
					parent_id: input.parent_id ?? null,
					position: (this.siblings(input.parent_id ?? null, '').at(-1)?.position ?? -1) + 1,
					title: input.title ?? '',
					icon: '',
					schema_version: 1,
					version: 1,
					access: 'manager',
					content: input.content ?? structuredClone(EMPTY),
					created_at: stamp,
					updated_at: stamp,
					trashed_at: null
				};
				this.state.documents.push(doc);
				return json(doc, 201);
			}
		],
		['GET', /^\/documents\/(?<id>[^/]+)$/, this.withDoc((doc) => json(doc))],
		[
			'PATCH',
			/^\/documents\/(?<id>[^/]+)$/,
			this.withDoc(async (doc, _p, request) => {
				Object.assign(doc, await body(request), { updated_at: this.now() });
				return json(doc);
			})
		],
		[
			'POST',
			/^\/documents\/(?<id>[^/]+)\/move$/,
			this.withDoc(async (doc, _p, request) => {
				const { parent_id = null, before = null } = await body(request);
				const row = this.siblings(parent_id, doc.id);
				const at = before === null ? row.length : row.findIndex((d) => d.id === before);
				const prev = row[at - 1]?.position;
				const following = row[at]?.position;
				doc.parent_id = parent_id;
				doc.position =
					prev === undefined ? (following ?? 1) - 1 : following === undefined ? prev + 1 : (prev + following) / 2;
				return json(doc);
			})
		],
		[
			'DELETE',
			/^\/documents\/(?<id>[^/]+)$/,
			this.withDoc((doc) => ((doc.trashed_at = this.now()), new Response(null, { status: 204 })))
		],
		['POST', /^\/documents\/(?<id>[^/]+)\/restore$/, this.withDoc((doc) => ((doc.trashed_at = null), json(doc)))],
		[
			'DELETE',
			/^\/documents\/(?<id>[^/]+)\/permanent$/,
			this.withDoc((doc) => {
				if (!doc.trashed_at) return fail(412, 'failed_precondition', 'document is not in the trash');
				this.state.documents = this.state.documents.filter((d) => d.id !== doc.id);
				return new Response(null, { status: 204 });
			})
		],
		['GET', /^\/documents\/(?<id>[^/]+)\/revisions$/, this.withDoc((doc) => json(this.state.revisions[doc.id] ?? []))],
		[
			'POST',
			/^\/documents\/(?<id>[^/]+)\/revisions\/(?<rev>[^/]+)\/restore$/,
			this.withDoc((doc, params) => {
				const revision = this.state.revisions[doc.id]?.find((r) => r.id === params.rev);
				return revision
					? json(
							Object.assign(doc, {
								title: revision.title,
								updated_at: this.now()
							})
						)
					: fail(404, 'not_found', 'revision not found');
			})
		],
		[
			'GET',
			/^\/documents\/(?<id>[^/]+)\/export\.(md|commonmark)$/,
			this.withDoc(
				(doc) =>
					new Response(`# ${doc.title}\n`, {
						headers: { 'Content-Type': 'text/markdown' }
					})
			)
		],
		[
			'POST',
			/^\/documents\/(?<id>[^/]+)\/attachments$/,
			this.withDoc(() => {
				const id = this.next();
				const src = `/api/attachments/${id}`;
				return json({ id, url: src, src, name: 'fichier', size: 0, mime: 'application/octet-stream' }, 201);
			})
		],
		[
			'GET',
			/^\/search$/,
			(_p, url) => {
				const q = (url.searchParams.get('q') ?? '').toLowerCase();
				const space = url.searchParams.get('space_id');
				const hits = this.state.documents.filter(
					(d) => d.space_id === space && !d.trashed_at && d.title.toLowerCase().includes(q)
				);
				return json(
					hits.map((d) => ({
						id: d.id,
						title: d.title,
						snippet: `<b>${d.title}</b>`,
						updated_at: d.updated_at
					}))
				);
			}
		],
		...shareRoutes(
			() => this.state,
			() => this.next(),
			(id) => this.find(id)
		),
		...actionRoutes(() => this.state)
	];

	async handle(request: Request): Promise<Response> {
		const url = new URL(request.url);
		const path = url.pathname.replace(/^\/api/, '');
		const method = request.method.toUpperCase();
		const csrf = request.headers.has('X-Facile-CSRF');
		this.state.calls.push({ method, path, csrf, body: null });
		if (!['GET', 'HEAD'].includes(method) && !csrf) return fail(403, 'permission_denied', 'missing CSRF header');
		if (!this.state.user && !path.startsWith('/public') && path !== '/auth/session')
			return fail(401, 'unauthenticated', 'sign in');
		for (const [verb, pattern, handler] of this.routes) {
			const match = verb === method ? pattern.exec(path) : null;
			if (match) return handler(match.groups ?? {}, url, request);
		}
		return fail(404, 'not_found', `no mock route for ${method} ${path}`);
	}
}
