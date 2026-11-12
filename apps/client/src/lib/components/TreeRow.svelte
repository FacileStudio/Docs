<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Icon, IconButton, icons, toast } from '@facile/muse';
	import { tree } from '$lib/tree.svelte';
	import { canPlace, dropPlacement, shiftPlacement, type DropZone, type Shift } from '$lib/tree-order';
	import type { Placement, TreeNode } from '$lib/types';
	import PageGlyph from './PageGlyph.svelte';
	import TreeRow from './TreeRow.svelte';

	let {
		node,
		depth = 0,
		manage = false,
		edit = false,
		oncontextmenu
	}: {
		node: TreeNode;
		depth?: number;
		manage?: boolean;
		edit?: boolean;
		oncontextmenu?: (e: MouseEvent, node: TreeNode) => void;
	} = $props();

	let zone = $state<DropZone | null>(null);

	const kids = $derived(tree.children(node.id));
	const open = $derived(Boolean(tree.expanded[node.id]));
	const active = $derived(page.url.pathname === `/d/${node.id}`);
	const keys: Record<string, Shift> = { ArrowUp: 'up', ArrowDown: 'down', ArrowRight: 'in', ArrowLeft: 'out' };

	function zoneAt(event: DragEvent): DropZone {
		const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
		const y = (event.clientY - box.top) / box.height;
		return y < 0.25 ? 'before' : y > 0.75 ? 'after' : 'inside';
	}

	async function place(id: string, placement: Placement) {
		try {
			await tree.move(id, placement);
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Déplacement impossible');
		}
	}

	function over(event: DragEvent) {
		const id = tree.dragging;
		if (!id || id === node.id) return;
		const next = zoneAt(event);
		if (!canPlace(tree.nodes, id, dropPlacement(tree.nodes, node.id, next))) return;
		event.preventDefault();
		zone = next;
	}

	function drop(event: DragEvent) {
		event.preventDefault();
		const id = tree.dragging;
		const at = zone;
		zone = null;
		tree.dragging = null;
		if (id && at) void place(id, dropPlacement(tree.nodes, node.id, at));
	}

	function shift(event: KeyboardEvent) {
		const move = keys[event.key];
		if (!manage || !event.altKey || !move) return;
		event.preventDefault();
		const placement = shiftPlacement(tree.nodes, node.id, move);
		if (placement) void place(node.id, placement);
	}

	async function addChild() {
		try {
			const created = await tree.create(node.id);
			await goto(`/d/${created.id}`);
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Création impossible');
		}
	}
</script>

<li role="treeitem" aria-expanded={kids.length ? open : undefined} aria-selected={active}>
	<div
		role="group"
		class="group relative flex min-h-9 items-center gap-1 rounded-fc-md pr-1 text-fc-sm transition-colors
			{active ? 'bg-fc-accent text-fc-accent-fg' : 'text-fc-fg-muted hover:bg-fc-surface hover:text-fc-fg'}
			{zone === 'inside' ? 'bg-fc-surface' : ''}"
		style:padding-left="{depth * 12 + 4}px"
		ondragover={over}
		ondragleave={() => (zone = null)}
		ondrop={drop}
		oncontextmenu={(event) => {
			event.preventDefault();
			event.stopPropagation();
			oncontextmenu?.(event, node);
		}}
	>
		{#if zone === 'before'}<span class="pointer-events-none absolute inset-x-1 top-0 h-0.5 rounded-fc-pill bg-fc-accent"
			></span>{/if}
		{#if zone === 'after'}<span
				class="pointer-events-none absolute inset-x-1 bottom-0 h-0.5 rounded-fc-pill bg-fc-accent"
			></span>{/if}
		<button
			type="button"
			class="flex size-6 shrink-0 items-center justify-center rounded-fc-sm {kids.length ? '' : 'invisible'}"
			aria-label={open ? 'Replier' : 'Déplier'}
			onclick={() => tree.toggle(node.id)}
		>
			<Icon icon={open ? icons.chevronDown : icons.arrow} size={14} />
		</button>
		<a
			href="/d/{node.id}"
			class="flex min-w-0 flex-1 items-center gap-2 py-1.5 outline-none"
			draggable={manage}
			aria-current={active ? 'page' : undefined}
			aria-keyshortcuts={manage ? 'Alt+ArrowUp Alt+ArrowDown Alt+ArrowLeft Alt+ArrowRight' : undefined}
			ondragstart={(event) => {
				tree.dragging = node.id;
				event.dataTransfer?.setData('text/plain', node.id);
				if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move';
			}}
			ondragend={() => (tree.dragging = null)}
			onkeydown={shift}
		>
			<PageGlyph value={node.icon} />
			<span class="truncate">{node.title || 'Sans titre'}</span>
		</a>
		{#if edit}
			<IconButton
				variant="ghost"
				class="size-7 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100"
				aria-label="Nouvelle sous-page"
				onclick={addChild}
			>
				<Icon icon={icons.plus} size={14} />
			</IconButton>
		{/if}
	</div>
	{#if open && kids.length}
		<ul role="group">
			{#each kids as child (child.id)}<TreeRow node={child} depth={depth + 1} {manage} {edit} {oncontextmenu} />{/each}
		</ul>
	{/if}
</li>
