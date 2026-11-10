<script lang="ts">
	import { Editor, type CollabUser } from '@facile/encre';
	import { toast } from '@facile/muse';
	import { onDestroy, untrack } from 'svelte';
	import { openCollab, type CollabEvents } from '$lib/collab';
	import { imageFiles, insertImages } from '$lib/images';

	let {
		documentId,
		editable,
		user,
		events
	}: {
		documentId: string;
		editable: boolean;
		user: CollabUser;
		events: CollabEvents;
	} = $props();

	const collab = untrack(() => openCollab(documentId, events));
	let editor: ReturnType<typeof Editor> | undefined = $state();

	onDestroy(() => collab.destroy());

	async function takeImages(event: ClipboardEvent | DragEvent) {
		const data = 'clipboardData' in event ? event.clipboardData : event.dataTransfer;
		const files = imageFiles(data);
		const tiptap = editor?.instance();
		if (!editable || !files.length || !tiptap) return;
		event.preventDefault();
		event.stopPropagation();
		try {
			await insertImages(tiptap, documentId, files);
		} catch (error) {
			toast.danger(error instanceof Error ? error.message : "L'image n'a pas pu être envoyée");
		}
	}
</script>

<div role="presentation" onpastecapture={takeImages} ondropcapture={takeImages}>
	<Editor
		bind:this={editor}
		ydoc={collab.ydoc}
		provider={collab.provider}
		{user}
		{editable}
		placeholder="Écrivez, ou tapez / pour insérer un bloc"
	/>
</div>
