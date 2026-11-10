import { afterEach, describe, expect, it, vi } from 'vitest';
import { backend } from './backend';
import { MockApi } from './dev/mock-api';
import { useMockFetch } from './dev/fetch';
import { absoluteUrl, createShare, withShareToken } from './share';

afterEach(() => vi.unstubAllGlobals());

describe('share flow', () => {
	it('creates a link, shows it absolute, serves it publicly without cookies, then revokes it', async () => {
		const api = new MockApi();
		const { inits } = useMockFetch(api);
		const parent = await backend.create({ title: 'Guide' });
		await backend.create({ title: 'Annexe', parent_id: parent.id });

		const link = await createShare(parent.id, true, 'https://docs.facile.studio');
		expect(link.url).toMatch(/^https:\/\/docs\.facile\.studio\/s\/tok-\d+$/);
		expect(api.state.shares[0]).toMatchObject({ document_id: parent.id, children: true });

		const token = link.url.split('/s/')[1];
		const shared = await backend.publicPage(token);
		expect(shared.title).toBe('Guide');
		expect(shared.children?.map((c) => c.title)).toEqual(['Annexe']);
		expect(inits.at(-1)?.credentials).toBe('omit');

		await backend.revokeShare(link.id);
		await expect(backend.publicPage(token)).rejects.toMatchObject({ code: 'not_found' });
	});

	it('keeps an absolute URL from the API as it is', () => {
		expect(absoluteUrl('https://docs.facile.studio/s/x', 'http://localhost:5173')).toBe(
			'https://docs.facile.studio/s/x'
		);
	});

	it('adds the share token to attachment images only', () => {
		const doc = {
			type: 'doc',
			content: [
				{ type: 'image', attrs: { src: '/api/attachments/a1', alt: 'a' } },
				{ type: 'image', attrs: { src: 'https://example.com/b.png', alt: 'b' } },
				{ type: 'paragraph', content: [{ type: 'text', text: 'x' }] }
			]
		};
		const out = withShareToken(doc, 't k');
		expect(out.content?.[0].attrs?.src).toBe('/api/attachments/a1?share=t%20k');
		expect(out.content?.[1].attrs?.src).toBe('https://example.com/b.png');
		expect(out.content?.[2]).toEqual(doc.content[2]);
	});
});
