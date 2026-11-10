<script lang="ts">
	import { Button, Icon, Popover } from '@facile/muse';
	import { documentIcon } from '$lib/brand';

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
	<span class="flex size-11 items-center justify-center text-2xl" aria-hidden="true">
		{#if value}{value}{:else}<Icon icon={documentIcon} size={24} />{/if}
	</span>
{:else}
	<Popover label="Icône du document" bind:open align="start">
		{#snippet trigger()}
			<span class="flex size-8 items-center justify-center text-2xl">
				{#if value}{value}{:else}<Icon icon={documentIcon} size={24} />{/if}
			</span>
		{/snippet}
		<div class="flex w-64 flex-col gap-2 p-2">
			<div class="grid grid-cols-5 gap-1">
				{#each choices as icon (icon)}
					<button
						type="button"
						class="flex size-11 items-center justify-center rounded-fc-md text-xl hover:bg-fc-surface {icon === value
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
