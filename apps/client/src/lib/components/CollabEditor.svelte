<script lang="ts">
	import { Blob } from '@facile/blob';
	import { Editor, httpsUrl, type AwarenessProvider, type ContextMenuInfo, type CollabUser } from '@facile/encre';
	import { toast } from '@facile/muse';
	import { onDestroy, untrack } from 'svelte';
	import { resolveEmbed } from '$lib/bulle';
	import { openCollab, type CollabEvents } from '$lib/collab';
	import { renderDiagram } from '$lib/diagrams';
	import { editorMenu } from '$lib/editor-menu';
	import { MAX_BYTES, fileUploader, uploader } from '$lib/images';
	import { EDITOR_COPY, browserLocale } from '$lib/locale';
	import ContextMenu, { type ContextMenuItem } from './ContextMenu.svelte';

	let {
		documentId,
		editable,
		user,
		events,
		assistant,
		onask,
		provider = $bindable()
	}: {
		documentId: string;
		editable: boolean;
		user: CollabUser;
		events: CollabEvents;
		/** The name of the person's Bulle assistant. */
		assistant: string;
		/** Hands the selection, as Facile Markdown, to the assistant. */
		onask: (markdown: string) => void;
		provider?: AwarenessProvider;
	} = $props();

	const collab = untrack(() => openCollab(documentId, events));
	const onimage = untrack(() => uploader(documentId));
	const onfile = untrack(() => fileUploader(documentId));

	const locale = browserLocale();
	const copy = EDITOR_COPY[locale];

	let editor = $state<ReturnType<typeof Editor>>();
	let menu = $state<{ x: number; y: number; items: ContextMenuItem[] } | null>(null);

	provider = collab.provider;

	onDestroy(() => {
		provider = undefined;
		collab.destroy();
	});

	function ask() {
		const markdown = editor?.selectionMarkdown() ?? '';
		if (markdown.trim()) onask(markdown);
	}

	function clipboard(cut: boolean) {
		editor?.instance()?.commands.focus();
		document.execCommand(cut ? 'cut' : 'copy');
	}

	async function paste() {
		const target = editor?.instance()?.view.dom;
		if (!target) return;
		try {
			const text = await navigator.clipboard.readText();
			const data = new DataTransfer();
			data.setData('text/plain', text);
			target.focus();
			target.dispatchEvent(new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true }));
		} catch {
			toast.warning('Le presse-papiers est fermé : collez avec Ctrl+V');
		}
	}

	function oncontextmenu(info: ContextMenuInfo) {
		if (!editor) return;
		const items = editorMenu({
			editor,
			editable,
			empty: info.empty,
			markdown: info.markdown,
			pageUrl: `${location.origin}/d/${documentId}`,
			assistant,
			onask,
			onpaste: () => void paste(),
			oncopy: clipboard,
			oncopied: () => toast.success('Lien du bloc copié')
		});
		menu = { x: info.event.clientX, y: info.event.clientY, items };
	}
</script>

<Editor
	bind:this={editor}
	ydoc={collab.ydoc}
	provider={collab.provider}
	presence="none"
	toolbar={false}
	{user}
	{editable}
	{onimage}
	{onfile}
	onembed={(url) => httpsUrl(url) !== null}
	{resolveEmbed}
	{oncontextmenu}
	{locale}
	{renderDiagram}
	maxImageBytes={MAX_BYTES}
	onerror={(error) => toast.danger(error.message || copy.uploadFailed)}
	placeholder={copy.placeholder}
>
	{#snippet extras()}
		<button
			type="button"
			class="flex h-8 shrink-0 items-center gap-1.5 rounded-fc-sm px-2 text-fc-sm whitespace-nowrap text-fc-fg hover:bg-fc-surface
				focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
			onmousedown={(event) => event.preventDefault()}
			onclick={ask}
		>
			<Blob name={assistant} size={18} />
			Demander à {assistant}
		</button>
	{/snippet}
</Editor>

{#if menu}
	<ContextMenu x={menu.x} y={menu.y} items={menu.items} onclose={() => (menu = null)} />
{/if}
