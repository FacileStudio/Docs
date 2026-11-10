import type { Placement, TreeNode } from './types';

/** Where a drop lands on a row: before it, inside it as its last child, or after it. */
export type DropZone = 'before' | 'inside' | 'after';

/** A keyboard move of one row. */
export type Shift = 'up' | 'down' | 'in' | 'out';

/** The children of `parentId`, ordered by position, then title for ties. */
export function siblings(nodes: readonly TreeNode[], parentId: string | null): TreeNode[] {
	return nodes
		.filter((node) => node.parent_id === parentId)
		.sort((a, b) => a.position - b.position || a.title.localeCompare(b.title));
}

/** Whether `id` is `ancestor` or sits anywhere below it. */
export function isWithin(nodes: readonly TreeNode[], id: string | null, ancestor: string): boolean {
	const parents = new Map(nodes.map((node) => [node.id, node.parent_id]));
	const seen = new Set<string>();
	for (let at = id; at !== null && !seen.has(at); at = parents.get(at) ?? null) {
		if (at === ancestor) return true;
		seen.add(at);
	}
	return false;
}

/** The position a node takes at `placement`: the midpoint of its new neighbours, as the server computes it. */
export function positionAt(nodes: readonly TreeNode[], id: string, placement: Placement): number {
	const row = siblings(nodes, placement.parent_id).filter((node) => node.id !== id);
	const index = placement.before === null ? row.length : row.findIndex((node) => node.id === placement.before);
	const at = index < 0 ? row.length : index;
	const prev = row[at - 1]?.position;
	const next = row[at]?.position;
	if (prev === undefined && next === undefined) return 0;
	if (prev === undefined) return (next as number) - 1;
	if (next === undefined) return prev + 1;
	return (prev + next) / 2;
}

/** The placement a drop on `targetId` in `zone` asks for. */
export function dropPlacement(nodes: readonly TreeNode[], targetId: string, zone: DropZone): Placement {
	const target = nodes.find((node) => node.id === targetId);
	if (!target || zone === 'inside') return { parent_id: targetId, before: null };
	if (zone === 'before') return { parent_id: target.parent_id, before: targetId };
	const row = siblings(nodes, target.parent_id);
	const next = row[row.findIndex((node) => node.id === targetId) + 1];
	return { parent_id: target.parent_id, before: next?.id ?? null };
}

/** The placement a keyboard shift asks for, or null when the row cannot move that way. */
export function shiftPlacement(nodes: readonly TreeNode[], id: string, shift: Shift): Placement | null {
	const node = nodes.find((item) => item.id === id);
	if (!node) return null;
	const row = siblings(nodes, node.parent_id);
	const index = row.findIndex((item) => item.id === id);
	if (shift === 'up') return index > 0 ? { parent_id: node.parent_id, before: row[index - 1].id } : null;
	if (shift === 'down')
		return index < row.length - 1 ? { parent_id: node.parent_id, before: row[index + 2]?.id ?? null } : null;
	if (shift === 'in') return index > 0 ? { parent_id: row[index - 1].id, before: null } : null;
	if (node.parent_id === null) return null;
	return dropPlacement(nodes, node.parent_id, 'after');
}

/** Whether `placement` is a legal move for `id`: never into itself or its own subtree. */
export function canPlace(nodes: readonly TreeNode[], id: string, placement: Placement): boolean {
	return placement.before !== id && !isWithin(nodes, placement.parent_id, id);
}
