<script lang="ts">
	import { Alert, Button, Modal, SecretField, SettingsRow, Switch, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { formatDate } from '$lib/format';
	import { createShare, type ShareLink } from '$lib/share';
	import type { ShareInfo } from '$lib/types';

	let { open = $bindable(false), documentId }: { open?: boolean; documentId: string } = $props();

	let children = $state(false);
	let link = $state<ShareLink | null>(null);
	let links = $state<ShareInfo[]>([]);
	let busy = $state(false);

	const failed = (error: unknown, fallback: string) => toast.danger(error instanceof Error ? error.message : fallback);

	async function refresh() {
		try {
			links = await backend.shares(documentId);
		} catch (error) {
			failed(error, 'Les liens n’ont pas pu être chargés');
		}
	}

	$effect(() => {
		if (!open) return;
		link = null;
		children = false;
		void refresh();
	});

	async function create() {
		busy = true;
		try {
			link = await createShare(documentId, children, window.location.origin);
			await refresh();
		} catch (error) {
			failed(error, 'Le lien n’a pas pu être créé');
		} finally {
			busy = false;
		}
	}

	async function revoke(id: string) {
		busy = true;
		try {
			await backend.revokeShare(id);
			if (link?.id === id) link = null;
			links = links.filter((item) => item.id !== id);
			toast.success('Lien révoqué');
		} catch (error) {
			failed(error, 'Le lien n’a pas pu être révoqué');
		} finally {
			busy = false;
		}
	}
</script>

<Modal {open} title="Partager" onClose={() => (open = false)}>
	<div class="flex flex-col gap-4">
		{#if link}
			<Alert tone="warning"
				>Copiez ce lien maintenant : il ne sera plus affiché. Toute personne qui l’a peut lire la page.</Alert
			>
			<SecretField value={link.url} sensitive={false} />
		{:else}
			<p class="text-fc-sm text-fc-fg-muted">Un lien public donne accès en lecture seule, sans compte Facile.</p>
			<SettingsRow label="Inclure les sous-pages" description="Le lien ouvre aussi toutes les pages sous celle-ci.">
				<Switch bind:checked={children} aria-label="Inclure les sous-pages" />
			</SettingsRow>
		{/if}
		{#if links.length}
			<section aria-label="Liens actifs" class="flex flex-col gap-2">
				<h3 class="text-fc-xs font-semibold tracking-wide text-fc-fg-muted uppercase">Liens actifs</h3>
				{#each links as item (item.id)}
					<div class="flex items-center gap-3 rounded-fc-md bg-fc-surface px-3 py-2">
						<span class="min-w-0 flex-1 text-fc-sm text-fc-fg">
							{item.children ? 'Page et sous-pages' : 'Page seule'}
							<span class="text-fc-xs text-fc-fg-muted">· {formatDate(item.created_at)}</span>
						</span>
						<Button variant="ghost-danger" size="sm" icon={icons.revoke} disabled={busy} onclick={() => revoke(item.id)}
							>Révoquer</Button
						>
					</div>
				{/each}
			</section>
		{/if}
	</div>
	{#snippet footer()}
		{#if link}
			<Button onclick={() => (open = false)}>Terminé</Button>
		{:else}
			<Button variant="ghost" onclick={() => (open = false)}>Fermer</Button>
			<Button disabled={busy} onclick={create}>Créer le lien</Button>
		{/if}
	{/snippet}
</Modal>
