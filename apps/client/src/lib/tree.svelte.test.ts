import { afterEach, describe, expect, it, vi } from 'vitest';
import { MockApi } from './dev/mock-api';
import { useMockFetch } from './dev/fetch';
import { DocumentTree } from './tree.svelte';

afterEach(() => vi.unstubAllGlobals());

async function seeded() {
	const api = new MockApi();
	useMockFetch(api);
	const tree = new DocumentTree();
	await tree.load(null);
	const a = await tree.create();
	const b = await tree.create();
	const child = await tree.create(a.id);
	return { api, tree, a, b, child };
}

describe('document tree', () => {
	it('loads the scope and lists recent documents first', async () => {
		const { tree, a, b, child } = await seeded();
		expect(tree.children(null).map((n) => n.id)).toEqual([a.id, b.id]);
		expect(tree.children(a.id).map((n) => n.id)).toEqual([child.id]);
		expect(tree.recent[0].id).toBe(child.id);
		expect(tree.expanded[a.id]).toBe(true);
	});

	it('moves a node, sends the placement and keeps the server position', async () => {
		const { api, tree, a, b } = await seeded();
		await tree.move(b.id, { parent_id: null, before: a.id });
		expect(tree.children(null).map((n) => n.id)).toEqual([b.id, a.id]);
		expect(api.state.calls.at(-1)).toMatchObject({ method: 'POST', path: `/documents/${b.id}/move`, csrf: true });
		expect(tree.nodes.find((n) => n.id === b.id)?.position).toBe(
			api.state.documents.find((d) => d.id === b.id)?.position
		);
	});

	it('refuses a move into its own subtree without calling the API', async () => {
		const { api, tree, a, child } = await seeded();
		const calls = api.state.calls.length;
		expect(await tree.move(a.id, { parent_id: child.id, before: null })).toBe(false);
		expect(api.state.calls.length).toBe(calls);
	});

	it('restores the previous order when the server refuses the move', async () => {
		const { api, tree, a, b } = await seeded();
		const before = tree.children(null).map((n) => n.id);
		api.state.documents = api.state.documents.filter((d) => d.id !== b.id);
		await expect(tree.move(b.id, { parent_id: null, before: a.id })).rejects.toMatchObject({ code: 'not_found' });
		expect(tree.children(null).map((n) => n.id)).toEqual(before);
	});

	it('removes a subtree', async () => {
		const { tree, a, b } = await seeded();
		tree.remove(a.id);
		expect(tree.nodes.map((n) => n.id)).toEqual([b.id]);
	});
});
