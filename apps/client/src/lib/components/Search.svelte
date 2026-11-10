<script lang="ts">
	import { goto } from '$app/navigation';
	import { Icon, Input, Modal, Spinner, icons } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { documentIcon } from '$lib/brand';
	import { session } from '$lib/session.svelte';
	import { snippetParts } from '$lib/snippet';
	import type { SearchHit } from '$lib/types';

	let { open = $bindable(false) }: { open?: boolean } = $props();

	let query = $state('');
	let hits = $state<SearchHit[]>([]);
	let selected = $state(0);
	let busy = $state(false);
	let failed = $state(false);

	$effect(() => {
		const q = query.trim();
		const spaceId = session.spaceId;
		if (!open || q.length < 2) {
			hits = [];
			return;
		}
		const abort = new AbortController();
		const timer = setTimeout(async () => {
			busy = true;
			try {
				hits = await backend.search(q, spaceId, abort.signal);
				failed = false;
				selected = 0;
			} catch {
				failed = !abort.signal.aborted;
			} finally {
				busy = false;
			}
		}, 200);
		return () => {
			clearTimeout(timer);
			abort.abort();
		};
	});

	function close() {
		open = false;
		query = '';
	}

	async function choose(hit: SearchHit | undefined) {
		if (!hit) return;
		close();
		await goto(`/d/${hit.id}`);
	}

	function navigate(event: KeyboardEvent) {
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			const step = event.key === 'ArrowDown' ? 1 : -1;
			selected = hits.length ? (selected + step + hits.length) % hits.length : 0;
		} else if (event.key === 'Enter') {
			event.preventDefault();
			void choose(hits[selected]);
		}
	}
</script>

<Modal {open} title="Rechercher" size="lg" onClose={close}>
	<div class="flex flex-col gap-3">
		<Input
			type="search"
			bind:value={query}
			placeholder="Titre ou contenu"
			aria-label="Rechercher dans les documents"
			aria-controls="search-results"
			aria-activedescendant={hits[selected] ? `hit-${hits[selected].id}` : undefined}
			autofocus
			onkeydown={navigate}
		/>
		{#if busy}
			<div class="flex justify-center py-4"><Spinner /></div>
		{:else if failed}
			<p class="py-4 text-center text-fc-sm text-fc-danger">La recherche a échoué. Réessayez.</p>
		{:else if query.trim().length >= 2 && hits.length === 0}
			<p class="py-4 text-center text-fc-sm text-fc-fg-muted">Aucun résultat.</p>
		{/if}
		<ul id="search-results" role="listbox" aria-label="Résultats" class="flex max-h-96 flex-col gap-1 overflow-y-auto">
			{#each hits as hit, i (hit.id)}
				<li id="hit-{hit.id}" role="option" aria-selected={i === selected}>
					<button
						type="button"
						class="flex w-full items-start gap-3 rounded-fc-md px-3 py-2.5 text-left
							{i === selected ? 'bg-fc-surface text-fc-fg' : 'text-fc-fg-muted hover:bg-fc-surface'}"
						onmouseenter={() => (selected = i)}
						onclick={() => choose(hit)}
					>
						<Icon icon={documentIcon} size={16} class="mt-0.5" />
						<span class="flex min-w-0 flex-col gap-0.5">
							<span class="truncate text-fc-sm font-medium text-fc-fg">{hit.title || 'Sans titre'}</span>
							<span class="line-clamp-2 text-fc-xs">
								{#each snippetParts(hit.snippet) as part, p (p)}
									{#if part.hit}<mark class="rounded-fc-sm bg-fc-accent/15 text-fc-fg">{part.text}</mark
										>{:else}{part.text}{/if}
								{/each}
							</span>
						</span>
					</button>
				</li>
			{/each}
		</ul>
		<p class="flex items-center gap-2 text-fc-xs text-fc-fg-muted">
			<Icon icon={icons.search} size={12} />Flèches pour choisir, Entrée pour ouvrir
		</p>
	</div>
</Modal>
