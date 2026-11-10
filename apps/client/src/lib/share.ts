import { backend } from './backend';
import type { TreeNode } from './types';

/** A share link just created: its id, to revoke it, and its full URL, shown once. */
export interface ShareLink {
	id: string;
	url: string;
}

/** The absolute URL of a share link, whether the API answers a path or a full URL. */
export function absoluteUrl(url: string, origin: string): string {
	return new URL(url, origin).toString();
}

/** Creates a public link to document `id`, with its subtree when `children` is set. */
export async function createShare(id: string, children: boolean, origin: string): Promise<ShareLink> {
	const created = await backend.share(id, children);
	return { id: created.id, url: absoluteUrl(created.url, origin) };
}

/** A shared subtree page with its depth under the shared page, for an indented list. */
export interface Outlined {
	node: TreeNode;
	depth: number;
}

/** The shared subtree depth first, siblings by position, from the flat list `GET /public/{token}` answers. */
export function outline(nodes: TreeNode[], rootId: string): Outlined[] {
	const byParent = Map.groupBy(nodes, (node) => node.parent_id ?? '');
	const walk = (parent: string, depth: number): Outlined[] =>
		[...(byParent.get(parent) ?? [])]
			.sort((a, b) => a.position - b.position)
			.flatMap((node) => [{ node, depth }, ...walk(node.id, depth + 1)]);
	return walk(rootId, 0);
}
