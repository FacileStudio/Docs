<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button, ConfirmModal, Icon, IconButton, Menu, StatusDot, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { shareIcon } from '$lib/brand';
	import type { Connection } from '$lib/collab';
	import { runExport, type ExportKind } from '$lib/export';
	import type { Access } from '$lib/session.svelte';
	import { tree } from '$lib/tree.svelte';
	import type { DocumentRecord } from '$lib/types';
	import IconChooser from './IconChooser.svelte';
	import ShareDialog from './ShareDialog.svelte';

	let {
		doc = $bindable(),
		access,
		connection,
		commonMark
	}: { doc: DocumentRecord; access: Access; connection: Connection; commonMark: () => Promise<string> } = $props();

	let title = $state('');
	let sharing = $state(false);
	let trashing = $state(false);

	$effect(() => {
		title = doc.title;
	});

	const status = $derived(
		{
			live: { tone: 'success', label: 'En ligne', pulse: false },
			connecting: { tone: 'warning', label: 'Reconnexion…', pulse: true },
			offline: { tone: 'danger', label: 'Hors ligne : vos modifications partiront au retour du réseau', pulse: false }
		}[connection] as { tone: 'success' | 'warning' | 'danger'; label: string; pulse: boolean }
	);

	async function save(patch: { title?: string; icon?: string }) {
		try {
			doc = await backend.update(doc.id, patch);
			tree.upsert(doc);
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Modification impossible');
		}
	}

	function commitTitle() {
		const next = title.trim();
		if (next && next !== doc.title) void save({ title: next });
		else title = doc.title;
	}

	async function trash() {
		trashing = false;
		try {
			await backend.trash(doc.id);
			tree.remove(doc.id);
			toast.success('Document mis à la corbeille');
			await goto('/');
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : 'Mise à la corbeille impossible');
		}
	}

	function exporting(kind: ExportKind) {
		return () => runExport(doc.id, doc.title, kind, commonMark).catch(() => toast.danger('Export impossible'));
	}

	const exports = $derived([
		{ label: 'PDF', icon: icons.download, onselect: exporting('pdf') },
		{ label: 'Markdown', icon: icons.download, onselect: exporting('markdown') },
		{ label: 'CommonMark', icon: icons.download, onselect: exporting('commonmark') }
	]);
</script>

<header class="flex flex-col gap-3">
	<div class="flex flex-wrap items-center justify-end gap-2">
		<StatusDot tone={status.tone} label={status.label} pulse={status.pulse} class="mr-auto" />
		{#if access === 'view'}<span class="text-fc-xs text-fc-fg-muted">Lecture seule</span>{/if}
		<Button variant="ghost" size="sm" icon={icons.history} href="/d/{doc.id}/history">Historique</Button>
		<Menu label="Exporter" items={exports} align="end">
			{#snippet trigger()}<span class="flex items-center gap-2"><Icon icon={icons.download} size={16} />Exporter</span
				>{/snippet}
		</Menu>
		{#if access === 'manage'}
			<IconButton variant="ghost" aria-label="Partager" onclick={() => (sharing = true)}>
				<Icon icon={shareIcon} size={18} />
			</IconButton>
			{#if !doc.trashed_at}
				<IconButton variant="ghost-danger" aria-label="Mettre à la corbeille" onclick={() => (trashing = true)}>
					<Icon icon={icons.remove} size={18} />
				</IconButton>
			{/if}
		{/if}
	</div>
	<div class="flex items-center gap-2">
		<IconChooser value={doc.icon} disabled={access === 'view'} onpick={(icon) => save({ icon })} />
		<input
			class="bare min-w-0 flex-1 bg-transparent text-3xl font-semibold text-fc-fg outline-none
				placeholder:text-fc-fg-muted"
			aria-label="Titre du document"
			placeholder="Sans titre"
			maxlength="200"
			readonly={access === 'view'}
			bind:value={title}
			onblur={commitTitle}
			onkeydown={(event) => event.key === 'Enter' && (event.currentTarget as HTMLInputElement).blur()}
		/>
	</div>
</header>

<ShareDialog bind:open={sharing} documentId={doc.id} />
<ConfirmModal
	open={trashing}
	tone="danger"
	title="Mettre à la corbeille ?"
	description="Le document et ses sous-pages quittent l’arbre. Un responsable peut les restaurer depuis la corbeille."
	confirmLabel="Mettre à la corbeille"
	onConfirm={trash}
	onCancel={() => (trashing = false)}
/>
