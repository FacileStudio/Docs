<script lang="ts">
	import { page } from '$app/state';
	import { Alert, Button, EmptyState, Spinner, icons, toast } from '@facile/muse';
	import { ApiError, backend } from '$lib/backend';
	import type { AwarenessProvider } from '@facile/encre';
	import type { CollabEvents, Connection, Stop } from '$lib/collab';
	import { assistant as askAssistant, DEFAULT_ASSISTANT, rememberSidebar, sidebarOpen } from '$lib/bulle';
	import BulleSidebar from '$lib/components/BulleSidebar.svelte';
	import CollabEditor from '$lib/components/CollabEditor.svelte';
	import DocumentHeader from '$lib/components/DocumentHeader.svelte';
	import { session } from '$lib/session.svelte';
	import { tree } from '$lib/tree.svelte';
	import type { DocumentRecord } from '$lib/types';

	const id = $derived(page.params.id ?? '');

	let doc = $state<DocumentRecord | null>(null);
	let missing = $state(false);
	let connection = $state<Connection>('connecting');
	let provider = $state<AwarenessProvider>();
	let stop = $state<Exclude<Stop, 'refused'> | null>(null);
	let generation = $state(0);
	let assistant = $state(DEFAULT_ASSISTANT.name);
	let bulleOpen = $state(false);
	let sidebar = $state<ReturnType<typeof BulleSidebar>>();

	const owner = $derived(session.user?.email ?? null);

	$effect(() => {
		void askAssistant().then((found) => (assistant = found.name));
	});

	$effect(() => {
		if (owner !== null) bulleOpen = sidebarOpen(owner);
	});

	function toggleBulle() {
		bulleOpen = !bulleOpen;
	}

	$effect(() => {
		if (owner !== null) rememberSidebar(owner, bulleOpen);
	});

	function ask(markdown: string) {
		sidebar?.ask(markdown);
	}

	const STOPPED: Record<Exclude<Stop, 'refused'>, string> = {
		'too-large': 'Le document dépasse la taille maximale : vos dernières modifications n’ont pas été enregistrées.',
		failed: 'Le document n’a pas pu être chargé sur le serveur.'
	};

	function load(wanted: string): Promise<void> {
		return backend
			.get(wanted)
			.then((record) => {
				if (wanted === id) doc = record;
			})
			.catch((error) => {
				if (wanted === id) missing = error instanceof ApiError && error.status === 404;
			});
	}

	$effect(() => {
		const wanted = id;
		doc = null;
		missing = false;
		stop = null;
		void load(wanted);
	});

	function reopen() {
		stop = null;
		connection = 'connecting';
		generation += 1;
	}

	async function refused() {
		await load(id);
		if (missing) return;
		reopen();
		toast.warning('Le document a refusé votre modification ; il a été rechargé.');
	}

	const events: CollabEvents = {
		onconnection: (state) => (connection = state),
		onstop: (why) => (why === 'refused' ? void refused() : (stop = why)),
		onroomclose: () => void load(id)
	};

	const access = $derived(doc?.access ?? 'viewer');
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

<div class="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 sm:px-6 md:py-10 {bulleOpen ? 'lg:mr-[400px]' : ''}">
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
				{#if access === 'manager'}<Button size="sm" variant="secondary" onclick={restore}>Restaurer</Button>{/if}
			</Alert>
		{/if}
		<DocumentHeader bind:doc {connection} {provider} {assistant} {bulleOpen} ontogglebulle={toggleBulle} />
		{#if stop}
			<Alert tone="danger">
				{STOPPED[stop]}
				<Button size="sm" variant="secondary" onclick={reopen}>Recharger</Button>
			</Alert>
		{/if}
		{#key `${doc.id}:${generation}`}
			<CollabEditor
				documentId={doc.id}
				editable={access !== 'viewer' && !doc.trashed_at}
				{user}
				{events}
				{assistant}
				onask={ask}
				bind:provider
			/>
		{/key}
		<BulleSidebar
			bind:this={sidebar}
			bind:open={bulleOpen}
			{assistant}
			page={{ app: 'docs', id: doc.id, title: doc.title, url: `${page.url.origin}/d/${doc.id}` }}
			onready={(name) => (assistant = name)}
		/>
	{/if}
</div>
