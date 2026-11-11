<script lang="ts">
	import { page } from '$app/state';
	import { Document } from '@facile/encre';
	import { EmptyState, Spinner, icons } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { renderDiagram } from '$lib/diagrams';
	import { glyph } from '$lib/format';
	import { browserLocale } from '$lib/locale';
	import { outline } from '$lib/share';
	import type { PublicPage } from '$lib/types';

	const locale = browserLocale();
	const token = $derived(page.params.token ?? '');
	const childId = $derived(page.url.searchParams.get('doc') ?? undefined);

	let root = $state<PublicPage | null>(null);
	let shown = $state<PublicPage | null>(null);
	let failed = $state(false);

	const subtree = $derived(root ? outline(root.children, root.id) : []);

	$effect(() => {
		const wanted = token;
		failed = false;
		backend
			.publicPage(wanted)
			.then((found) => (root = found))
			.catch(() => (failed = true));
	});

	$effect(() => {
		const wanted = childId;
		if (!wanted) {
			shown = root;
			return;
		}
		shown = null;
		backend
			.publicPage(token, wanted)
			.then((found) => (shown = found))
			.catch(() => (failed = true));
	});
</script>

<svelte:head>
	<meta name="robots" content="noindex, nofollow" />
	<meta name="referrer" content="no-referrer" />
	<title>{shown?.title || 'Document partagé'}</title>
</svelte:head>

<div class="min-h-dvh bg-fc-bg">
	<div class="mx-auto flex w-full max-w-3xl flex-col gap-8 px-4 py-10 sm:px-6 md:py-16">
		{#if failed}
			<EmptyState icon={icons.warning} title="Lien invalide" description="Ce lien a été révoqué ou n’existe pas." />
		{:else if !shown}
			<div class="flex justify-center py-16"><Spinner /></div>
		{:else}
			{#if childId && root}
				<a href="/s/{token}" class="text-fc-sm text-fc-fg-muted hover:text-fc-fg">← {root.title || 'Retour'}</a>
			{/if}
			<h1 class="text-3xl font-semibold text-fc-fg">{shown.title || 'Sans titre'}</h1>
			<Document doc={shown.content} {locale} {renderDiagram} />
			{#if subtree.length}
				<nav aria-label="Sous-pages" class="flex flex-col gap-1">
					<h2 class="text-fc-xs font-semibold tracking-wide text-fc-fg-muted uppercase">Sous-pages</h2>
					{#each subtree as { node, depth } (node.id)}
						<a
							href="/s/{token}?doc={node.id}"
							aria-current={node.id === childId ? 'page' : undefined}
							style:padding-left="{0.75 + depth * 1.25}rem"
							class="rounded-fc-md py-2 pr-3 text-fc-sm text-fc-fg hover:bg-fc-surface aria-[current=page]:bg-fc-surface"
						>
							{glyph(node.icon)}
							{node.title || 'Sans titre'}
						</a>
					{/each}
				</nav>
			{/if}
		{/if}
	</div>
</div>
