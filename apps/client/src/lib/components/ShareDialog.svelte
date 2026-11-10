<script lang="ts">
	import { Alert, Button, Modal, SecretField, SettingsRow, Switch, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { createShare, type ShareLink } from '$lib/share';

	let { open = $bindable(false), documentId }: { open?: boolean; documentId: string } = $props();

	let children = $state(false);
	let link = $state<ShareLink | null>(null);
	let busy = $state(false);

	$effect(() => {
		if (open) {
			link = null;
			children = false;
		}
	});

	async function create() {
		busy = true;
		try {
			link = await createShare(documentId, children, window.location.origin);
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Le lien n’a pas pu être créé');
		} finally {
			busy = false;
		}
	}

	async function revoke() {
		if (!link) return;
		busy = true;
		try {
			await backend.revokeShare(link.id);
			link = null;
			toast.success('Lien révoqué');
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Le lien n’a pas pu être révoqué');
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
	</div>
	{#snippet footer()}
		{#if link}
			<Button variant="ghost-danger" icon={icons.revoke} disabled={busy} onclick={revoke}>Révoquer</Button>
			<Button onclick={() => (open = false)}>Terminé</Button>
		{:else}
			<Button variant="ghost" onclick={() => (open = false)}>Annuler</Button>
			<Button disabled={busy} onclick={create}>Créer le lien</Button>
		{/if}
	{/snippet}
</Modal>
