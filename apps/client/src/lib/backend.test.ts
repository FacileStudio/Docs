import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, backend, request } from './backend';

function stub(response: Response | Error) {
	const fetch = vi.fn(async () => {
		if (response instanceof Error) throw response;
		return response;
	});
	vi.stubGlobal('fetch', fetch);
	return fetch;
}

const sent = (fetch: ReturnType<typeof stub>) => fetch.mock.calls[0] as unknown as [string, RequestInit];

afterEach(() => vi.unstubAllGlobals());

describe('request', () => {
	it('sends cookies on every call and the CSRF header only on mutations', async () => {
		const fetch = stub(Response.json({ ok: true }));
		await request('/spaces');
		const [url, init] = sent(fetch);
		expect(url).toBe('/api/spaces');
		expect(init.credentials).toBe('include');
		expect(new Headers(init.headers).has('X-Facile-CSRF')).toBe(false);

		fetch.mockResolvedValueOnce(Response.json({}));
		await request('/documents', { method: 'post', body: '{}' });
		const headers = new Headers((fetch.mock.calls[1] as unknown as [string, RequestInit])[1].headers);
		expect(headers.get('X-Facile-CSRF')).toBe('1');
		expect(headers.get('Content-Type')).toBe('application/json');
	});

	it('leaves the multipart content type to the browser', async () => {
		const fetch = stub(Response.json({ id: 'a', url: '/api/attachments/a' }, { status: 201 }));
		await backend.upload('d1', new File(['x'], 'x.png', { type: 'image/png' }));
		const headers = new Headers(sent(fetch)[1].headers);
		expect(headers.has('Content-Type')).toBe(false);
		expect(headers.get('X-Facile-CSRF')).toBe('1');
	});

	it('throws the suite envelope as a typed ApiError', async () => {
		stub(Response.json({ error: { code: 'not_found', message: 'document not found' } }, { status: 404 }));
		const error = await request('/documents/x').catch((e: unknown) => e);
		expect(error).toBeInstanceOf(ApiError);
		expect(error).toMatchObject({
			status: 404,
			code: 'not_found',
			message: 'document not found'
		});
	});

	it('maps the status to a code when the body is not the envelope', async () => {
		stub(new Response('<html>', { status: 429, statusText: 'Too Many Requests' }));
		await expect(request('/search?q=a')).rejects.toMatchObject({
			status: 429,
			code: 'rate_limited'
		});
	});

	it('reports a network failure as code network', async () => {
		stub(new TypeError('Failed to fetch'));
		await expect(request('/spaces')).rejects.toMatchObject({
			status: 0,
			code: 'network'
		});
	});

	it('answers undefined for 204', async () => {
		stub(new Response(null, { status: 204 }));
		await expect(backend.trash('d1')).resolves.toBeUndefined();
	});

	it('never sends the session cookie to the public share routes', async () => {
		const fetch = stub(Response.json({ title: 't', content: { type: 'doc' } }));
		await backend.publicPage('tok', 'child');
		expect(sent(fetch)).toEqual(['/api/public/tok/documents/child', expect.objectContaining({ credentials: 'omit' })]);
	});
});
