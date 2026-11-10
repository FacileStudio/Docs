import type { EncreNode } from '@facile/encre';
import type {
	Created,
	DocumentRecord,
	Placement,
	PublicPage,
	Revision,
	SearchHit,
	SessionUser,
	Space,
	TreeNode
} from './types';

/** The suite error codes, from tronc's `statusByCode`. */
export type ErrorCode =
	| 'invalid_argument'
	| 'unauthenticated'
	| 'permission_denied'
	| 'not_found'
	| 'already_exists'
	| 'failed_precondition'
	| 'resource_exhausted'
	| 'rate_limited'
	| 'method_not_allowed'
	| 'unavailable'
	| 'internal'
	| 'network';

const CODES: Record<number, ErrorCode> = {
	400: 'invalid_argument',
	401: 'unauthenticated',
	403: 'permission_denied',
	404: 'not_found',
	405: 'method_not_allowed',
	409: 'already_exists',
	412: 'failed_precondition',
	413: 'resource_exhausted',
	429: 'rate_limited',
	503: 'unavailable'
};

const SAFE = new Set(['GET', 'HEAD', 'OPTIONS']);

/** The API base. Same origin: the Vite proxy in development, the Rust binary in production. */
export const API = '/api';

/** A failed API call, carrying the suite envelope `{"error":{"code","message"}}`. */
export class ApiError extends Error {
	constructor(
		readonly status: number,
		readonly code: ErrorCode,
		message: string
	) {
		super(message);
		this.name = 'ApiError';
	}
}

/** Reads the suite error envelope of a failed response into an `ApiError`. */
export async function toApiError(response: Response): Promise<ApiError> {
	const fallback = CODES[response.status] ?? 'internal';
	try {
		const body = (await response.json()) as { error?: { code?: string; message?: string } };
		const code = (body.error?.code as ErrorCode | undefined) ?? fallback;
		return new ApiError(response.status, code, body.error?.message ?? response.statusText);
	} catch {
		return new ApiError(response.status, fallback, response.statusText || `HTTP ${response.status}`);
	}
}

/** Calls the API: cookies included, the CSRF header on every mutation, the error envelope thrown typed. */
export async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
	const method = (init.method ?? 'GET').toUpperCase();
	const headers = new Headers(init.headers);
	if (typeof init.body === 'string' && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json');
	if (!SAFE.has(method)) headers.set('X-Facile-CSRF', '1');
	let response: Response;
	try {
		response = await fetch(`${API}${path}`, { credentials: 'include', ...init, method, headers });
	} catch {
		throw new ApiError(0, 'network', 'Network error');
	}
	if (!response.ok) throw await toApiError(response);
	if (response.status === 204) return undefined as T;
	return (await response.json()) as T;
}

/** A list the plan writes as a bare array; tolerates a `{key: [...]}` wrapper as Feuilles answers. */
export function list<T>(value: unknown, key: string): T[] {
	if (Array.isArray(value)) return value as T[];
	const wrapped = (value as Record<string, unknown> | null)?.[key];
	return Array.isArray(wrapped) ? (wrapped as T[]) : [];
}

const send = (method: string, body?: unknown): RequestInit => ({
	method,
	body: body === undefined ? undefined : JSON.stringify(body)
});

const scoped = (spaceId: string | null, extra: Record<string, string> = {}) => {
	const params = new URLSearchParams(extra);
	if (spaceId) params.set('space_id', spaceId);
	const query = params.toString();
	return query ? `?${query}` : '';
};

const doc = (id: string, rest = '') => `/documents/${encodeURIComponent(id)}${rest}`;

/** Every call the client makes, one method per route of plan section 8.3. */
export const backend = {
	session: () => request<{ user: SessionUser | null }>('/auth/session').then((r) => r.user),
	loginUrl: () => `${API}/auth/oidc`,
	logout: () => request<unknown>('/auth/logout', send('POST')),
	spaces: () => request<unknown>('/spaces').then((r) => list<Space>(r, 'spaces')),
	tree: (spaceId: string | null) =>
		request<unknown>(`/documents${scoped(spaceId)}`).then((r) => list<TreeNode>(r, 'documents')),
	trashed: (spaceId: string | null) =>
		request<unknown>(`/documents${scoped(spaceId, { trashed: 'true' })}`).then((r) => list<TreeNode>(r, 'documents')),
	create: (input: { title: string; space_id?: string; parent_id?: string; content?: EncreNode; markdown?: string }) =>
		request<DocumentRecord>('/documents', send('POST', input)),
	get: (id: string) => request<DocumentRecord>(doc(id)),
	update: (id: string, patch: { title?: string; icon?: string }) =>
		request<DocumentRecord>(doc(id), send('PATCH', patch)),
	move: (id: string, placement: Placement) => request<DocumentRecord>(doc(id, '/move'), send('POST', placement)),
	trash: (id: string) => request<void>(doc(id), send('DELETE')),
	restore: (id: string) => request<DocumentRecord>(doc(id, '/restore'), send('POST')),
	destroy: (id: string) => request<void>(doc(id, '/permanent'), send('DELETE')),
	revisions: (id: string) => request<unknown>(doc(id, '/revisions')).then((r) => list<Revision>(r, 'revisions')),
	restoreRevision: (id: string, rev: string) =>
		request<DocumentRecord>(doc(id, `/revisions/${encodeURIComponent(rev)}/restore`), send('POST')),
	search: (q: string, spaceId: string | null, signal?: AbortSignal) =>
		request<unknown>(`/search${scoped(spaceId, { q })}`, { signal }).then((r) => list<SearchHit>(r, 'results')),
	share: (id: string, children: boolean) => request<Created>(doc(id, '/share'), send('POST', { children })),
	revokeShare: (shareId: string) => request<void>(`/share/${encodeURIComponent(shareId)}`, send('DELETE')),
	upload: (id: string, file: File) => {
		const form = new FormData();
		form.append('file', file);
		return request<Created>(doc(id, '/attachments'), { method: 'POST', body: form });
	},
	exportUrl: (id: string, kind: 'md' | 'pdf') => `${API}${doc(id, `/export.${kind}`)}`,
	publicPage: (token: string, childId?: string) => {
		const base = `/public/${encodeURIComponent(token)}`;
		const path = childId ? `${base}/documents/${encodeURIComponent(childId)}` : base;
		return request<PublicPage>(path, { credentials: 'omit' });
	}
};
