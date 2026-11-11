<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button, Card, EmptyState, icons, toast } from '@facile/muse';
	import { documentIcon } from '$lib/brand';
	import { formatDate } from '$lib/format';
	import { importMarkdown } from '$lib/import';
	import { session } from '$lib/session.svelte';
	import { tree } from '$lib/tree.svelte';
	import PageGlyph from '$lib/components/PageGlyph.svelte';

	let picker: HTMLInputElement | undefined = $state();
	let busy = $state(false);

	const recent = $derived(tree.recent.slice(0, 20));
	const edit = $derived(session.scopeAccess(session.spaceId) !== 'viewer');

	async function run(work: () => Promise<{ id: string }>) {
		busy = true;
		try {
			const created = await work();
			await goto(`/d/${created.id}`);
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Opération impossible');
		} finally {
			busy = false;
		}
	}

	function imported(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file) return;
		void run(async () => {
			const record = await importMarkdown(file, session.spaceId);
			tree.upsert(record);
			return record;
		});
	}
</script>

<svelte:head><title>Docs</title></svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 sm:px-6 md:py-10">
	<header class="flex flex-wrap items-start justify-between gap-4">
		<div class="flex flex-col gap-1">
			<h1 class="text-fc-2xl font-semibold whitespace-nowrap text-fc-fg">Documents récents</h1>
			<p class="text-fc-sm text-fc-fg-muted">{session.space?.name ?? 'Espace personnel'}</p>
		</div>
		{#if edit}
			<div class="flex flex-wrap items-center gap-2">
				<Button variant="ghost" icon={icons.upload} disabled={busy} onclick={() => picker?.click()}
					>Importer du markdown</Button
				>
				<Button icon={icons.plus} disabled={busy} onclick={() => run(() => tree.create())}>Nouveau document</Button>
			</div>
		{/if}
	</header>
	<input
		bind:this={picker}
		type="file"
		accept=".md,.markdown,text/markdown,text/plain"
		class="hidden"
		onchange={imported}
	/>

	{#if recent.length === 0 && !tree.loading}
		<EmptyState
			icon={documentIcon}
			title="Aucun document"
			description="Créez une première page, ou importez un fichier markdown."
		/>
	{:else}
		<ul class="flex flex-col gap-2">
			{#each recent as node (node.id)}
				<li>
					<a
						href="/d/{node.id}"
						class="block rounded-fc-lg outline-none focus-visible:ring-2 focus-visible:ring-fc-accent"
					>
						<Card class="flex items-center gap-3 transition-colors hover:bg-fc-surface">
							<PageGlyph value={node.icon} size={20} />
							<span class="min-w-0 flex-1 truncate text-fc-sm font-medium text-fc-fg">{node.title || 'Sans titre'}</span
							>
							<span class="shrink-0 text-fc-xs text-fc-fg-muted">{formatDate(node.updated_at)}</span>
						</Card>
					</a>
				</li>
			{/each}
		</ul>
	{/if}
</div>
