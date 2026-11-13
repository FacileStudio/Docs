<script lang="ts">
	import { untrack } from 'svelte';
	import { BULLE_ORIGIN, bulleMessage, embedUrl, trustedOrigin, type PageContext } from '$lib/bulle';

	let {
		open = $bindable(),
		page,
		assistant,
		onready
	}: {
		open: boolean;
		page: PageContext;
		assistant: string;
		/** Bulle named the assistant once its frame loaded. */
		onready?: (name: string) => void;
	} = $props();

	let frame = $state<HTMLIFrameElement | null>(null);
	let ready = $state(false);
	let queued: string[] = [];
	let shown = $state(false);

	const src = $derived.by(() => {
		const id = page.id;
		return untrack(() => embedUrl({ ...page, id }));
	});
	const dev = import.meta.env.DEV;

	$effect(() => {
		if (open) shown = true;
	});

	$effect(() => {
		void src;
		ready = false;
	});

	function post(text: string) {
		const message = { type: 'bulle:ask', text, context: { ...page } };
		frame?.contentWindow?.postMessage(message, BULLE_ORIGIN);
	}

	/** Opens the panel and asks the assistant about `text`, once its frame is ready. */
	export function ask(text: string): void {
		open = true;
		if (ready) post(text);
		else queued.push(text);
	}

	function onmessage(event: MessageEvent) {
		if (!trustedOrigin(event.origin, dev) || event.source !== frame?.contentWindow) return;
		const message = bulleMessage(event.data);
		if (message?.type === 'bulle:close') open = false;
		if (message?.type !== 'bulle:ready') return;
		ready = true;
		onready?.(message.assistant.name);
		for (const text of queued.splice(0)) post(text);
	}
</script>

<svelte:window
	{onmessage}
	onkeydown={(event) => {
		if (open && event.key === 'Escape' && !event.defaultPrevented) open = false;
	}}
/>

{#if shown}
	<aside
		class="fixed inset-0 z-40 flex flex-col bg-fc-bg shadow-lg lg:inset-y-0 lg:right-0 lg:left-auto lg:w-[400px]"
		class:hidden={!open}
		aria-label={assistant}
		data-testid="bulle-sidebar"
	>
		<iframe bind:this={frame} title={assistant} {src} class="min-h-0 w-full flex-1 bg-fc-bg" allow="clipboard-write"
		></iframe>
	</aside>
{/if}
