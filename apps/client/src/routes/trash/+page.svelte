<script lang="ts">
	import { Button, Card, ConfirmModal, EmptyState, PageHeader, Spinner, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { formatDate } from '$lib/format';
	import { session } from '$lib/session.svelte';
	import { tree } from '$lib/tree.svelte';
	import type { TreeNode } from '$lib/types';

	let rows = $state<TreeNode[] | null>(null);
	let doomed = $state<TreeNode | null>(null);

	const lost =
		'Le document, ses sous-pages, ses versions et ses pièces jointes disparaissent. ' +
		'Les liens de partage cessent de fonctionner. Rien ne peut être récupéré.';

	const manage = $derived(session.accessTo(session.spaceId) === 'manage');

	$effect(() => {
		const spaceId = session.spaceId;
		rows = null;
		backend
			.trashed(spaceId)
			.then((list) => (rows = list))
			.catch((error) => {
				rows = [];
				toast.danger(error instanceof Error ? error.message : 'Corbeille indisponible');
			});
	});

	async function restore(row: TreeNode) {
		try {
			tree.upsert(await backend.restore(row.id));
			rows = rows?.filter((item) => item.id !== row.id) ?? null;
			toast.success('Document restauré');
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Restauration impossible');
		}
	}

	async function destroy() {
		const row = doomed;
		doomed = null;
		if (!row) return;
		try {
			await backend.destroy(row.id);
			rows = rows?.filter((item) => item.id !== row.id) ?? null;
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Suppression impossible');
		}
	}
</script>

<svelte:head><title>Corbeille · Docs</title></svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 sm:px-6 md:py-10">
	<PageHeader title="Corbeille" description="Un document mis à la corbeille emporte ses sous-pages." />
	{#if rows === null}
		<div class="flex justify-center py-16"><Spinner /></div>
	{:else if rows.length === 0}
		<EmptyState icon={icons.remove} title="La corbeille est vide" />
	{:else}
		<ul class="flex flex-col gap-2">
			{#each rows as row (row.id)}
				<li>
					<Card class="flex flex-wrap items-center gap-3">
						<div class="flex min-w-0 flex-1 flex-col">
							<span class="truncate text-fc-sm font-medium text-fc-fg">{row.icon} {row.title || 'Sans titre'}</span>
							<span class="text-fc-xs text-fc-fg-muted">{formatDate(row.trashed_at ?? row.updated_at)}</span>
						</div>
						{#if manage}
							<Button variant="secondary" size="sm" icon={icons.refresh} onclick={() => restore(row)}>Restaurer</Button>
							<Button variant="ghost-danger" size="sm" icon={icons.remove} onclick={() => (doomed = row)}
								>Supprimer</Button
							>
						{/if}
					</Card>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<ConfirmModal
	open={doomed !== null}
	tone="danger"
	title="Supprimer définitivement ?"
	description={lost}
	confirmLabel="Supprimer"
	onConfirm={destroy}
	onCancel={() => (doomed = null)}
/>
