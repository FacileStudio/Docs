<script lang="ts">
	import { Editor, type AwarenessProvider, type CollabUser } from '@facile/encre';
	import { toast } from '@facile/muse';
	import { onDestroy, untrack } from 'svelte';
	import { openCollab, type CollabEvents } from '$lib/collab';
	import { renderDiagram } from '$lib/diagrams';
	import { MAX_BYTES, uploader } from '$lib/images';
	import { EDITOR_COPY, browserLocale } from '$lib/locale';

	let {
		documentId,
		editable,
		user,
		events,
		provider = $bindable()
	}: {
		documentId: string;
		editable: boolean;
		user: CollabUser;
		events: CollabEvents;
		provider?: AwarenessProvider;
	} = $props();

	const collab = untrack(() => openCollab(documentId, events));
	const onimage = untrack(() => uploader(documentId));

	const locale = browserLocale();
	const copy = EDITOR_COPY[locale];

	provider = collab.provider;

	onDestroy(() => {
		provider = undefined;
		collab.destroy();
	});
</script>

<Editor
	ydoc={collab.ydoc}
	provider={collab.provider}
	presence="none"
	{user}
	{editable}
	{onimage}
	{locale}
	{renderDiagram}
	maxImageBytes={MAX_BYTES}
	onerror={(error) => toast.danger(error.message || copy.uploadFailed)}
	placeholder={copy.placeholder}
/>
