import { backend } from './backend';
import { canPlace, positionAt, siblings } from './tree-order';
import type { DocumentRecord, Placement, TreeNode } from './types';

/** The document tree of the active scope: rows, expanded branches, and moves applied optimistically. */
export class DocumentTree {
	nodes = $state<TreeNode[]>([]);
	expanded = $state<Record<string, boolean>>({});
	loading = $state(false);
	scope = $state<string | null>(null);
	dragging = $state<string | null>(null);

	recent = $derived([...this.nodes].sort((a, b) => b.updated_at.localeCompare(a.updated_at)));

	children(parentId: string | null): TreeNode[] {
		return siblings(this.nodes, parentId);
	}

	async load(spaceId: string | null): Promise<void> {
		this.scope = spaceId;
		this.loading = true;
		try {
			const nodes = await backend.tree(spaceId);
			if (this.scope === spaceId) this.nodes = nodes;
		} finally {
			this.loading = false;
		}
	}

	upsert(record: TreeNode | DocumentRecord): void {
		const row: TreeNode = {
			id: record.id,
			parent_id: record.parent_id,
			position: record.position,
			title: record.title,
			icon: record.icon,
			updated_at: record.updated_at
		};
		const at = this.nodes.findIndex((node) => node.id === row.id);
		this.nodes = at < 0 ? [...this.nodes, row] : this.nodes.map((node, i) => (i === at ? row : node));
		if (row.parent_id) this.expanded[row.parent_id] = true;
	}

	remove(id: string): void {
		const gone = new Set([id]);
		let grew = true;
		while (grew) {
			const before = gone.size;
			this.nodes.forEach((node) => node.parent_id && gone.has(node.parent_id) && gone.add(node.id));
			grew = gone.size > before;
		}
		this.nodes = this.nodes.filter((node) => !gone.has(node.id));
	}

	async move(id: string, placement: Placement): Promise<boolean> {
		if (!canPlace(this.nodes, id, placement)) return false;
		const previous = this.nodes;
		const position = positionAt(this.nodes, id, placement);
		this.nodes = this.nodes.map((node) =>
			node.id === id ? { ...node, parent_id: placement.parent_id, position } : node
		);
		if (placement.parent_id) this.expanded[placement.parent_id] = true;
		try {
			this.upsert(await backend.move(id, placement));
			return true;
		} catch (error) {
			this.nodes = previous;
			throw error;
		}
	}

	async create(parentId?: string): Promise<DocumentRecord> {
		const record = await backend.create({
			title: 'Sans titre',
			...(this.scope ? { space_id: this.scope } : {}),
			...(parentId ? { parent_id: parentId } : {})
		});
		this.upsert(record);
		return record;
	}

	toggle(id: string): void {
		this.expanded[id] = !this.expanded[id];
	}
}

/** The one tree of the app. */
export const tree = new DocumentTree();
