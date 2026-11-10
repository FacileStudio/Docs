import { afterEach, describe, expect, it, vi } from 'vitest';
import { backend } from './backend';
import { MockApi } from './dev/mock-api';
import { useMockFetch } from './dev/fetch';
import { absoluteUrl, createShare, outline } from './share';

afterEach(() => vi.unstubAllGlobals());

describe('share flow', () => {
	it('creates a link, shows it absolute, serves it publicly without cookies, then revokes it', async () => {
		const api = new MockApi();
		const { inits } = useMockFetch(api);
		const parent = await backend.create({ title: 'Guide' });
		await backend.create({ title: 'Annexe', parent_id: parent.id });

		const link = await createShare(parent.id, true, 'https://docs.facile.studio');
		expect(link.url).toMatch(/^https:\/\/docs\.facile\.studio\/s\/tok-mock-\d+$/);
		expect(api.state.shares[0]).toMatchObject({
			document_id: parent.id,
			children: true
		});

		const token = link.url.split('/s/')[1];
		const shared = await backend.publicPage(token);
		expect(shared.title).toBe('Guide');
		expect(shared.children.map((c) => c.title)).toEqual(['Annexe']);
		expect(inits.at(-1)?.credentials).toBe('omit');
		expect(await backend.shares(parent.id)).toEqual([expect.objectContaining({ id: link.id, children: true })]);

		await backend.revokeShare(link.id);
		expect(await backend.shares(parent.id)).toEqual([]);
		await expect(backend.publicPage(token)).rejects.toMatchObject({
			code: 'not_found'
		});
	});

	it('keeps an absolute URL from the API as it is', () => {
		expect(absoluteUrl('https://docs.facile.studio/s/x', 'http://localhost:5173')).toBe(
			'https://docs.facile.studio/s/x'
		);
	});
});

describe('outline', () => {
	it('lists the shared subtree depth first, siblings by position', () => {
		const node = (id: string, parent_id: string, position: number) => ({
			id,
			parent_id,
			position,
			title: id,
			icon: '',
			updated_at: ''
		});
		const nodes = [node('b', 'r', 2), node('a1', 'a', 1), node('a', 'r', 1), node('a2', 'a', 0)];
		expect(outline(nodes, 'r').map(({ node, depth }) => `${depth}${node.id}`)).toEqual(['0a', '1a2', '1a1', '0b']);
	});
});
