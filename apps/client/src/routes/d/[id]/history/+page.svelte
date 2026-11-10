<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Button, Card, ConfirmModal, EmptyState, PageHeader, Spinner, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { formatDate } from '$lib/format';
	import { session } from '$lib/session.svelte';
	import { tree } from '$lib/tree.svelte';
	import type { DocumentRecord, Revision } from '$lib/types';

	const id = $derived(page.params.id ?? '');

	let doc = $state<DocumentRecord | null>(null);
	let revisions = $state<Revision[] | null>(null);
	let pending = $state<Revision | null>(null);

	$effect(() => {
		const wanted = id;
		revisions = null;
		Promise.all([backend.get(wanted), backend.revisions(wanted)])
			.then(([record, list]) => {
				doc = record;
				revisions = list;
			})
			.catch((error) => {
				revisions = [];
				toast.danger(error instanceof Error ? error.message : 'Historique indisponible');
			});
	});

	const canRestore = $derived(doc ? session.accessTo(doc.space_id) !== 'view' : false);

	async function restore() {
		const revision = pending;
		pending = null;
		if (!revision) return;
		try {
			tree.upsert(await backend.restoreRevision(id, revision.id));
			toast.success('Version restaurée');
			await goto(`/d/${id}`);
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Restauration impossible');
		}
	}
</script>

<svelte:head><title>Historique · Docs</title></svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 sm:px-6 md:py-10">
	<PageHeader title="Historique" description={doc?.title ?? ''}>
		{#snippet actions()}<Button variant="ghost" icon={icons.chevronLeft} href="/d/{id}">Retour au document</Button
			>{/snippet}
	</PageHeader>
	{#if revisions === null}
		<div class="flex justify-center py-16"><Spinner /></div>
	{:else if revisions.length === 0}
		<EmptyState
			icon={icons.history}
			title="Aucune version"
			description="Une version est enregistrée toutes les dix minutes d’activité."
		/>
	{:else}
		<ul class="flex flex-col gap-2">
			{#each revisions as revision, i (revision.id)}
				<li>
					<Card class="flex items-center justify-between gap-3">
						<div class="flex min-w-0 flex-col">
							<span class="truncate text-fc-sm font-medium text-fc-fg">{revision.title || 'Sans titre'}</span>
							<span class="text-fc-xs text-fc-fg-muted"
								>{formatDate(revision.created_at)}{i === 0 ? ' · la plus récente' : ''}</span
							>
						</div>
						{#if canRestore}
							<Button variant="secondary" size="sm" icon={icons.refresh} onclick={() => (pending = revision)}
								>Restaurer</Button
							>
						{/if}
					</Card>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<ConfirmModal
	open={pending !== null}
	title="Restaurer cette version ?"
	description="Tout le monde voit le contenu de cette version. La version actuelle reste dans l’historique."
	confirmLabel="Restaurer"
	onConfirm={restore}
	onCancel={() => (pending = null)}
/>
