<script lang="ts">
	import { Icon } from '@facile/muse';
	import { pageIcon } from '$lib/icons';

	let {
		value,
		size = 16,
		class: className = ''
	}: { value: string | null | undefined; size?: number; class?: string } = $props();

	const shown = $derived(pageIcon(value));
</script>

{#if 'emoji' in shown}
	<span
		class="flex shrink-0 items-center justify-center overflow-hidden leading-none {className}"
		style:width="{size}px"
		style:height="{size}px"
		style:font-size="{Math.round(size * 0.9)}px"
		aria-hidden="true">{shown.emoji}</span
	>
{:else}
	<Icon icon={shown.glyph} {size} class={className} aria-hidden="true" />
{/if}
