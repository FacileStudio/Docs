<script lang="ts">
	import { goto } from '$app/navigation';
	import { MediaQuery } from 'svelte/reactivity';
	import { Blob } from '@facile/blob';
	import { caretColor, peers, type AwarenessProvider } from '@facile/encre';
	import {
		Button,
		ConfirmModal,
		ConnectionHint,
		Icon,
		IconButton,
		Menu,
		Presence,
		Tooltip,
		icons,
		toast,
		type PresencePeer
	} from '@facile/muse';
	import { backend } from '$lib/backend';
	import { shareIcon } from '$lib/brand';
	import type { Connection } from '$lib/collab';
	import { runExport, type ExportKind } from '$lib/export';
	import { tree } from '$lib/tree.svelte';
	import type { DocumentRecord } from '$lib/types';
	import IconChooser from './IconChooser.svelte';
	import ShareDialog from './ShareDialog.svelte';

	let {
		doc = $bindable(),
		connection,
		provider,
		assistant,
		bulleOpen,
		ontogglebulle
	}: {
		doc: DocumentRecord;
		connection: Connection;
		provider?: AwarenessProvider;
		/** The name of the person's Bulle assistant. */
		assistant: string;
		bulleOpen: boolean;
		ontogglebulle: () => void;
	} = $props();

	let others = $state<PresencePeer[]>([]);

	$effect(() => {
		if (!provider) return;
		const source = provider;
		const refresh = () => {
			const self = source.awareness.clientID;
			others = peers(source)
				.filter((peer) => peer.clientId !== self)
				.map((peer) => ({
					id: peer.clientId,
					name: peer.name,
					color: peer.color ?? caretColor(peer.clientId),
					picture: peer.avatar
				}));
		};
		refresh();
		source.awareness.on('change', refresh);
		return () => {
			source.awareness.off('change', refresh);
			others = [];
		};
	});

	const access = $derived(doc.access);
	const locked = $derived(access === 'viewer' || doc.trashed_at !== null);

	let title = $state('');
	let sharing = $state(false);
	let trashing = $state(false);

	$effect(() => {
		title = doc.title;
	});

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
		return () => runExport(doc.id, kind);
	}

	const exports = $derived([
		{ label: 'PDF', icon: icons.download, onselect: exporting('pdf') },
		{ label: 'Markdown', icon: icons.download, onselect: exporting('markdown') },
		{ label: 'CommonMark', icon: icons.download, onselect: exporting('commonmark') }
	]);

	const narrow = new MediaQuery('max-width: 639.98px');
</script>

<header class="flex flex-col gap-3" data-connection={connection}>
	<div class="flex flex-wrap items-center gap-x-2 gap-y-1">
		<div class="flex min-w-0 flex-1 items-center gap-2">
			<ConnectionHint state={connection} />
			<Presence peers={others} />
			{#if access === 'viewer'}<span class="text-fc-xs whitespace-nowrap text-fc-fg-muted">Lecture seule</span>{/if}
		</div>
		<div class="ml-auto flex shrink-0 items-center gap-1">
			<Tooltip text="Ouvrir {assistant}" describe={false} disabled={!narrow.current} class="contents">
				<button
					type="button"
					class="flex min-h-11 items-center gap-2 rounded-fc-md px-2 text-fc-sm text-fc-fg hover:bg-fc-surface
						focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring
						{bulleOpen ? 'bg-fc-surface' : ''}"
					aria-pressed={bulleOpen}
					aria-label="Ouvrir {assistant}"
					data-testid="bulle-toggle"
					onclick={ontogglebulle}
				>
					<Blob name={assistant} size={24} />
					<span class="hidden sm:inline">{assistant}</span>
				</button>
			</Tooltip>
			<Tooltip text="Historique" describe={false} disabled={!narrow.current} class="contents">
				<Button variant="ghost" size="sm" icon={icons.history} href="/d/{doc.id}/history" aria-label="Historique"
					><span class="hidden sm:inline">Historique</span></Button
				>
			</Tooltip>
			<Menu label="Exporter" items={exports} align="end">
				{#snippet trigger()}<span class="flex items-center gap-2"
						><Icon icon={icons.download} size={16} /><span class="hidden sm:inline">Exporter</span></span
					>{/snippet}
			</Menu>
			{#if access === 'manager'}
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
	</div>
	<div class="flex items-start gap-2">
		<IconChooser value={doc.icon} disabled={locked} onpick={(icon) => save({ icon })} />
		<div class="grid min-w-0 flex-1 text-2xl leading-tight font-semibold sm:text-3xl">
			<span aria-hidden="true" class="invisible col-start-1 row-start-1 py-1 break-words whitespace-pre-wrap"
				>{title || 'Sans titre'}&#32;</span
			>
			<textarea
				rows="1"
				class="col-start-1 row-start-1 resize-none overflow-hidden bg-transparent py-1 text-fc-fg outline-none
					placeholder:text-fc-fg-muted"
				aria-label="Titre du document"
				placeholder="Sans titre"
				maxlength="200"
				readonly={locked}
				bind:value={title}
				onblur={commitTitle}
				onkeydown={(event) => {
					if (event.key !== 'Enter') return;
					event.preventDefault();
					(event.currentTarget as HTMLTextAreaElement).blur();
				}}></textarea>
		</div>
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
