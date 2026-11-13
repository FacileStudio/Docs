<script lang="ts">
	import { Button, Tooltip } from '@facile/muse';
	import { PICKABLE } from '$lib/icons';
	import PageGlyph from './PageGlyph.svelte';

	let {
		value,
		disabled = false,
		onpick
	}: { value: string; disabled?: boolean; onpick: (icon: string) => void } = $props();

	let open = $state(false);
	let root = $state<HTMLElement | null>(null);

	$effect(() => {
		if (!open) return;
		const outside = (event: PointerEvent) => {
			if (root && event.target instanceof Node && !root.contains(event.target)) open = false;
		};
		const escape = (event: KeyboardEvent) => {
			if (event.key === 'Escape') open = false;
		};
		window.addEventListener('pointerdown', outside, true);
		window.addEventListener('keydown', escape);
		return () => {
			window.removeEventListener('pointerdown', outside, true);
			window.removeEventListener('keydown', escape);
		};
	});

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
	<span class="flex size-11 shrink-0 items-center justify-center" data-testid="page-icon"
		><PageGlyph {value} size={28} /></span
	>
{:else}
	<div bind:this={root} class="relative shrink-0">
		<Tooltip text="Icône du document" describe={false} disabled={open} class="contents">
			<button
				type="button"
				class="flex size-11 cursor-pointer items-center justify-center rounded-fc-md text-fc-fg hover:bg-fc-surface
					focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
				aria-label="Icône du document"
				aria-haspopup="dialog"
				aria-expanded={open}
				data-testid="page-icon"
				onclick={() => (open = !open)}
			>
				<PageGlyph {value} size={28} />
			</button>
		</Tooltip>
		{#if open}
			<div
				role="dialog"
				aria-label="Icône du document"
				class="absolute top-full left-0 z-30 mt-1 flex w-64 flex-col gap-2 rounded-fc-md bg-fc-bg p-2 text-fc-fg shadow-lg"
			>
				<div class="grid grid-cols-5 gap-1">
					{#each PICKABLE as choice (choice.value)}
						<Tooltip text={choice.label} describe={false} class="contents">
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
						</Tooltip>
					{/each}
					{#each choices as icon (icon)}
						<button
							type="button"
							class="flex size-11 cursor-pointer items-center justify-center rounded-fc-md text-xl hover:bg-fc-surface {icon ===
							value
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
		{/if}
	</div>
{/if}
