<script lang="ts">
	import { Button, Popover } from '@facile/muse';
	import { PICKABLE } from '$lib/icons';
	import PageGlyph from './PageGlyph.svelte';

	let {
		value,
		disabled = false,
		onpick
	}: { value: string; disabled?: boolean; onpick: (icon: string) => void } = $props();

	let open = $state(false);

	const choices = [
		'📄',
		'📝',
		'📚',
		'📌',
		'📎',
		'🗂️',
		'📊',
		'📈',
		'💡',
		'✅',
		'⚠️',
		'🚀',
		'🔒',
		'🧭',
		'🛠️',
		'🎯',
		'📅',
		'💬',
		'🌱',
		'⭐'
	];

	function pick(icon: string) {
		open = false;
		onpick(icon);
	}
</script>

{#if disabled}
	<span class="flex size-11 items-center justify-center"><PageGlyph {value} size={28} /></span>
{:else}
	<Popover label="Icône du document" bind:open align="start">
		{#snippet trigger()}
			<span class="flex size-8 items-center justify-center"><PageGlyph {value} size={28} /></span>
		{/snippet}
		<div class="flex w-64 flex-col gap-2 p-2">
			<div class="grid grid-cols-5 gap-1">
				{#each PICKABLE as choice (choice.value)}
					<button
						type="button"
						class="flex size-11 cursor-pointer items-center justify-center rounded-fc-md text-fc-fg hover:bg-fc-surface {choice.value ===
						value
							? 'bg-fc-surface'
							: ''}"
						aria-label="Choisir {choice.label}"
						aria-pressed={choice.value === value}
						onclick={() => pick(choice.value)}
					>
						<PageGlyph value={choice.value} size={20} />
					</button>
				{/each}
				{#each choices as icon (icon)}
					<button
						type="button"
						class="flex size-11 cursor-pointer items-center justify-center rounded-fc-md text-xl hover:bg-fc-surface {icon === value
							? 'bg-fc-surface'
							: ''}"
						aria-label="Choisir {icon}"
						aria-pressed={icon === value}
						onclick={() => pick(icon)}
					>
						{icon}
					</button>
				{/each}
			</div>
			{#if value}<Button variant="ghost" size="sm" onclick={() => pick('')}>Retirer l'icône</Button>{/if}
		</div>
	</Popover>
{/if}
