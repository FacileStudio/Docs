<script lang="ts">
	import { goto } from '$app/navigation';
	import { Icon, IconButton, Skeleton, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { session } from '$lib/session.svelte';
	import { tree } from '$lib/tree.svelte';
	import type { TreeNode } from '$lib/types';
	import ContextMenu, { type ContextMenuItem } from './ContextMenu.svelte';
	import TreeRow from './TreeRow.svelte';

	const roots = $derived(tree.children(null));
	const access = $derived(session.scopeAccess(session.spaceId));
	const manage = $derived(access === 'manager');
	const edit = $derived(access !== 'viewer');

	let rootOver = $state(false);
	let contextMenu = $state<{ x: number; y: number; node: TreeNode } | null>(null);

	async function create() {
		try {
			const created = await tree.create();
			await goto(`/d/${created.id}`);
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Création impossible');
		}
	}

	async function dropAtRoot(event: DragEvent) {
		event.preventDefault();
		rootOver = false;
		const id = tree.dragging;
		tree.dragging = null;
		if (!id) return;
		try {
			await tree.move(id, { parent_id: null, before: null });
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Déplacement impossible');
		}
	}

	async function createSubPage(parentId: string) {
		try {
			const created = await tree.create(parentId);
			await goto(`/d/${created.id}`);
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Création impossible');
		}
	}

	async function copyLink(docId: string) {
		try {
			if (navigator.clipboard?.writeText) {
				await navigator.clipboard.writeText(`${window.location.origin}/d/${docId}`);
			}
			toast.success('Lien copié');
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Copie impossible');
		}
	}

	async function trashDocument(docId: string) {
		try {
			await backend.trash(docId);
			tree.remove(docId);
			toast.success('Document placé dans la corbeille');
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Suppression impossible');
		}
	}

	function makeItems(node: TreeNode): ContextMenuItem[] {
		return [
			{
				label: 'Ouvrir',
				href: `/d/${node.id}`,
				icon: icons.arrow
			},
			{
				label: 'Nouvelle sous-page',
				icon: icons.plus,
				onselect: () => void createSubPage(node.id)
			},
			{
				label: 'Copier le lien',
				icon: icons.copy,
				onselect: () => void copyLink(node.id)
			},
			{ separator: true },
			{
				label: 'Supprimer',
				icon: icons.remove,
				tone: 'danger',
				onselect: () => void trashDocument(node.id)
			}
		];
	}

	const menuItems = $derived(contextMenu ? makeItems(contextMenu.node) : []);
</script>

<div class="flex h-full min-h-0 flex-col gap-2 p-3">
	<div class="flex items-center justify-between px-2">
		<span class="text-fc-xs font-semibold tracking-wide text-fc-fg-muted uppercase">
			{session.space?.name ?? 'Personnel'}
		</span>
		{#if edit}
			<IconButton variant="ghost" aria-label="Nouveau document" onclick={create}>
				<Icon icon={icons.plus} size={16} />
			</IconButton>
		{/if}
	</div>
	<div class="min-h-0 flex-1 overflow-y-auto overscroll-contain">
		{#if tree.loading && roots.length === 0}
			<div class="flex flex-col gap-2 px-2">
				<Skeleton class="h-6" /><Skeleton class="h-6" /><Skeleton class="h-6" />
			</div>
		{:else if roots.length === 0}
			<p class="px-2 py-4 text-fc-sm text-fc-fg-muted">Aucun document pour l'instant.</p>
		{:else}
			<ul role="tree" aria-label="Documents" class="flex flex-col gap-0.5">
				{#each roots as node (node.id)}
					<TreeRow
						{node}
						{manage}
						{edit}
						oncontextmenu={(event, targetNode) => {
							contextMenu = { x: event.clientX, y: event.clientY, node: targetNode };
						}}
					/>
				{/each}
			</ul>
		{/if}
		{#if manage && tree.dragging}
			<div
				role="region"
				aria-label="Déposer à la racine"
				class="mt-2 rounded-fc-md px-3 py-3 text-fc-xs text-fc-fg-muted {rootOver ? 'bg-fc-surface' : ''}"
				ondragover={(event) => {
					event.preventDefault();
					rootOver = true;
				}}
				ondragleave={() => (rootOver = false)}
				ondrop={dropAtRoot}
			>
				Déposer ici pour placer à la racine
			</div>
		{/if}
	</div>
</div>

{#if contextMenu}
	<ContextMenu
		x={contextMenu.x}
		y={contextMenu.y}
		items={menuItems}
		onclose={() => (contextMenu = null)}
	/>
{/if}
