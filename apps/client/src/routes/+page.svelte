<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button, Card, EmptyState, Icon, Menu, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { documentIcon } from '$lib/brand';
	import { formatDate } from '$lib/format';
	import { icons } from '$lib/icons';
	import { importMarkdown } from '$lib/import';
	import { session } from '$lib/session.svelte';
	import { tree } from '$lib/tree.svelte';
	import type { TreeNode } from '$lib/types';
	import ContextMenu, { type ContextMenuItem } from '$lib/components/ContextMenu.svelte';
	import DocumentPreview from '$lib/components/DocumentPreview.svelte';

	let picker: HTMLInputElement | undefined = $state();
	let busy = $state(false);
	let contextMenu = $state<{ x: number; y: number; node: TreeNode } | null>(null);
	let gridEl = $state<HTMLElement | null>(null);
	let cols = $state(1);

	const recent = $derived(tree.recent.slice(0, 20));
	const edit = $derived(session.scopeAccess(session.spaceId) !== 'viewer');

	function updateCols() {
		if (!gridEl) return;
		const style = window.getComputedStyle(gridEl).getPropertyValue('grid-template-columns');
		cols = style ? style.split(' ').filter(Boolean).length : 1;
	}

	$effect(() => {
		if (!gridEl) return;
		updateCols();
		const ro = new ResizeObserver(updateCols);
		ro.observe(gridEl);
		return () => ro.disconnect();
	});

	const showAddCard = $derived(cols > 1 && recent.length % cols !== 0 && edit);

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

	function actions(node: TreeNode): ContextMenuItem[] {
		return [
			{
				label: 'Ouvrir',
				href: `/d/${node.id}`,
				icon: icons.arrow,
				onselect: () => goto(`/d/${node.id}`)
			},
			...(edit
				? [
						{
							label: 'Nouvelle sous-page',
							icon: icons.plus,
							onselect: () => void run(() => tree.create(node.id))
						}
					]
				: []),
			{
				label: 'Copier le lien',
				icon: icons.copy,
				onselect: () => void copyLink(node.id)
			},
			...(edit
				? [
						{ separator: true as const },
						{
							label: 'Supprimer',
							icon: icons.remove,
							tone: 'danger' as const,
							onselect: () => void trashDocument(node.id)
						}
					]
				: [])
		];
	}
</script>

<svelte:head><title>Docs</title></svelte:head>

<div class="mx-auto flex w-full max-w-fc-xl flex-col gap-8 px-4 py-8 sm:px-6 md:px-10 md:py-10 pb-24 md:pb-32">
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
		<ul
			bind:this={gridEl}
			class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
			aria-label="Documents récents"
		>
			{#each recent as node (node.id)}
				<li
					class="group/doc relative"
					oncontextmenu={(e) => {
						e.preventDefault();
						contextMenu = { x: e.clientX, y: e.clientY, node };
					}}
				>
					<Card
						href="/d/{node.id}"
						class="flex h-full cursor-pointer flex-col gap-3 p-3 transition-[background-color,translate] duration-300 hover:-translate-y-1 motion-reduce:transition-none"
					>
						<DocumentPreview icon={node.icon} />
						<span class="flex min-w-0 flex-col gap-1 px-1 pb-1">
							<span class="truncate text-fc-sm font-medium text-fc-fg">{node.title || 'Sans titre'}</span>
							<span class="text-fc-xs text-fc-fg-muted">Modifié le {formatDate(node.updated_at)}</span>
						</span>
					</Card>
					<Menu
						label="Actions pour {node.title || 'Sans titre'}"
						items={actions(node)}
						align="end"
						class="absolute right-5 top-5 rounded-fc-md shadow-sm transition-opacity focus-within:opacity-100 group-hover/doc:opacity-100 motion-reduce:transition-none [@media(hover:hover)_and_(pointer:fine)]:opacity-0"
					>
						{#snippet trigger()}<Icon icon={icons.more} size={18} />{/snippet}
					</Menu>
				</li>
			{/each}
			{#if showAddCard}
				<li class="h-full">
					<button
						type="button"
						onclick={() => run(() => tree.create())}
						disabled={busy}
						class="group flex h-full min-h-[220px] w-full cursor-pointer flex-col items-center justify-center gap-3 rounded-fc-lg bg-fc-surface/50 p-4 text-center transition-[background-color,translate] duration-300 hover:-translate-y-1 hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring disabled:cursor-not-allowed motion-reduce:transition-none"
						aria-label="Nouveau document"
					>
						<span
							class="flex size-10 items-center justify-center rounded-full bg-fc-component text-fc-fg-muted transition-colors group-hover:bg-fc-page group-hover:text-fc-fg"
						>
							<Icon icon={icons.plus} size={20} />
						</span>
						<span class="flex flex-col gap-0.5">
							<span class="text-fc-sm font-medium text-fc-fg">Nouveau document</span>
							<span class="text-fc-xs text-fc-fg-muted">Créer une page</span>
						</span>
					</button>
				</li>
			{/if}
		</ul>
	{/if}
</div>

{#if contextMenu}
	<ContextMenu
		x={contextMenu.x}
		y={contextMenu.y}
		items={actions(contextMenu.node)}
		onclose={() => (contextMenu = null)}
	/>
{/if}
