<script lang="ts">
	import { page } from '$app/state';
	import { Alert, Button, EmptyState, Spinner, icons } from '@facile/muse';
	import { ApiError, backend } from '$lib/backend';
	import type { Connection } from '$lib/collab';
	import CollabEditor from '$lib/components/CollabEditor.svelte';
	import DocumentHeader from '$lib/components/DocumentHeader.svelte';
	import { session } from '$lib/session.svelte';
	import { tree } from '$lib/tree.svelte';
	import type { DocumentRecord } from '$lib/types';

	const id = $derived(page.params.id ?? '');

	let doc = $state<DocumentRecord | null>(null);
	let missing = $state(false);
	let connection = $state<Connection>('connecting');
	let editor: CollabEditor | undefined = $state();

	$effect(() => {
		const wanted = id;
		doc = null;
		missing = false;
		backend
			.get(wanted)
			.then((record) => wanted === id && (doc = record))
			.catch((error) => wanted === id && (missing = error instanceof ApiError && error.status === 404));
	});

	const access = $derived(doc ? session.accessTo(doc.space_id) : 'view');
	const user = $derived({
		name: session.user?.name || session.user?.email || 'Anonyme',
		avatar: session.user?.avatar_url
	});

	async function restore() {
		if (!doc) return;
		doc = await backend.restore(doc.id);
		tree.upsert(doc);
	}
</script>

<svelte:head><title>{doc?.title || 'Document'} · Docs</title></svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 sm:px-6 md:py-10">
	{#if missing}
		<EmptyState
			icon={icons.warning}
			title="Document introuvable"
			description="Il a été supprimé, ou vous n’y avez pas accès."
		>
			<Button href="/">Retour à l’accueil</Button>
		</EmptyState>
	{:else if !doc}
		<div class="flex justify-center py-16"><Spinner /></div>
	{:else}
		{#if doc.trashed_at}
			<Alert tone="warning">
				Ce document est dans la corbeille.
				{#if access === 'manage'}<Button size="sm" variant="secondary" onclick={restore}>Restaurer</Button>{/if}
			</Alert>
		{/if}
		<DocumentHeader
			bind:doc
			{access}
			{connection}
			commonMark={() => editor?.exportCommonMark() ?? Promise.resolve('')}
		/>
		{#key doc.id}
			<CollabEditor
				bind:this={editor}
				documentId={doc.id}
				editable={access !== 'view' && !doc.trashed_at}
				{user}
				onconnection={(state) => (connection = state)}
			/>
		{/key}
	{/if}
</div>
