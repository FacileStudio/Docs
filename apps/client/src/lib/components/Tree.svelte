<script lang="ts">
	import { goto } from '$app/navigation';
	import { Icon, IconButton, Skeleton, icons, toast } from '@facile/muse';
	import { session } from '$lib/session.svelte';
	import { tree } from '$lib/tree.svelte';
	import TreeRow from './TreeRow.svelte';

	const roots = $derived(tree.children(null));
	const access = $derived(session.scopeAccess(session.spaceId));
	const manage = $derived(access === 'manager');
	const edit = $derived(access !== 'viewer');

	let rootOver = $state(false);

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
				{#each roots as node (node.id)}<TreeRow {node} {manage} {edit} />{/each}
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
