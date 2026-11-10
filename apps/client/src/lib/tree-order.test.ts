import { describe, expect, it } from 'vitest';
import { canPlace, dropPlacement, isWithin, positionAt, shiftPlacement, siblings } from './tree-order';
import type { TreeNode } from './types';

const node = (id: string, parent_id: string | null, position: number): TreeNode => ({
	id,
	parent_id,
	position,
	title: id,
	icon: '',
	updated_at: '2026-11-10T00:00:00Z'
});

const nodes = [
	node('a', null, 0),
	node('b', null, 1),
	node('c', null, 2),
	node('a1', 'a', 0),
	node('a2', 'a', 5),
	node('a1x', 'a1', 0)
];

describe('tree order', () => {
	it('orders siblings by position', () => {
		expect(siblings([node('z', null, 3), node('y', null, -1)], null).map((n) => n.id)).toEqual(['y', 'z']);
	});

	it('puts a moved node at the midpoint of its new neighbours', () => {
		expect(positionAt(nodes, 'c', { parent_id: null, before: 'b' })).toBe(0.5);
		expect(positionAt(nodes, 'a1x', { parent_id: 'a', before: 'a2' })).toBe(2.5);
	});

	it('goes one step past the edge at either end and to 0 in an empty parent', () => {
		expect(positionAt(nodes, 'c', { parent_id: null, before: 'a' })).toBe(-1);
		expect(positionAt(nodes, 'a', { parent_id: null, before: null })).toBe(3);
		expect(positionAt(nodes, 'b', { parent_id: 'c', before: null })).toBe(0);
	});

	it('ignores the moved node itself when it stays among its siblings', () => {
		expect(positionAt(nodes, 'a', { parent_id: null, before: 'c' })).toBe(1.5);
	});

	it('turns a drop zone into a placement', () => {
		expect(dropPlacement(nodes, 'b', 'before')).toEqual({ parent_id: null, before: 'b' });
		expect(dropPlacement(nodes, 'b', 'after')).toEqual({ parent_id: null, before: 'c' });
		expect(dropPlacement(nodes, 'c', 'after')).toEqual({ parent_id: null, before: null });
		expect(dropPlacement(nodes, 'b', 'inside')).toEqual({ parent_id: 'b', before: null });
	});

	it('moves with the keyboard: up, down, indent and outdent', () => {
		expect(shiftPlacement(nodes, 'b', 'up')).toEqual({ parent_id: null, before: 'a' });
		expect(shiftPlacement(nodes, 'a', 'down')).toEqual({ parent_id: null, before: 'c' });
		expect(shiftPlacement(nodes, 'b', 'down')).toEqual({ parent_id: null, before: null });
		expect(shiftPlacement(nodes, 'b', 'in')).toEqual({ parent_id: 'a', before: null });
		expect(shiftPlacement(nodes, 'a1', 'out')).toEqual({ parent_id: null, before: 'b' });
		expect(shiftPlacement(nodes, 'a', 'up')).toBeNull();
		expect(shiftPlacement(nodes, 'c', 'down')).toBeNull();
		expect(shiftPlacement(nodes, 'a', 'in')).toBeNull();
		expect(shiftPlacement(nodes, 'a', 'out')).toBeNull();
	});

	it('refuses a move into the node itself or its subtree', () => {
		expect(isWithin(nodes, 'a1x', 'a')).toBe(true);
		expect(isWithin(nodes, 'b', 'a')).toBe(false);
		expect(canPlace(nodes, 'a', { parent_id: 'a1x', before: null })).toBe(false);
		expect(canPlace(nodes, 'a', { parent_id: 'a', before: null })).toBe(false);
		expect(canPlace(nodes, 'a', { parent_id: null, before: 'a' })).toBe(false);
		expect(canPlace(nodes, 'a1x', { parent_id: null, before: 'a' })).toBe(true);
	});
});
