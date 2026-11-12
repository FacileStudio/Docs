<script lang="ts">
	import PageGlyph from './PageGlyph.svelte';

	let { icon }: { icon?: string | null } = $props();
</script>

<div
	class="relative flex aspect-[16/10] flex-col justify-between overflow-hidden rounded-fc-md bg-fc-surface p-4"
	aria-hidden="true"
>
	<div>
		<PageGlyph value={icon} size={22} />
	</div>
	<div class="flex flex-col gap-1.5">
		<div class="h-1.5 w-[70%] rounded-fc-pill bg-fc-component"></div>
		<div class="h-1.5 w-[90%] rounded-fc-pill bg-fc-component"></div>
		<div class="h-1.5 w-[55%] rounded-fc-pill bg-fc-component"></div>
	</div>
</div>
