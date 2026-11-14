<script lang="ts">
	import { Cursor, Stage, Tile, cursorAt, grow, type Act } from '@facile/mise-en-scene';
	import { Button, Icon, icons } from '@facile/muse';

	const REVISIONS = [
		{ title: 'Planning d’hiver', date: '14 nov. 2026, 10:40 · la plus récente' },
		{ title: 'Planning d’hiver', date: '14 nov. 2026, 10:20' }
	];

	let restored = $state(false);
	let cursor = $state(cursorAt());
	let target = $state<HTMLElement>();

	function reset() {
		restored = false;
		cursor.visible = false;
	}

	function poster() {
		reset();
		restored = true;
	}

	async function script(act: Act) {
		await act.wait(900);
		cursor.visible = true;
		await act.move(cursor, act.point(target, 0.55, 0.6));
		await act.click(cursor, () => (restored = true));
		await act.wait(300);
		cursor.visible = false;
		await act.wait(2600);
		reset();
		await act.wait(600);
	}
</script>

<Tile
	title="Revenez à une version passée"
	body="Une version toutes les dix minutes d’activité. Restaurez-la d’un clic."
>
	<Stage width={358} height={256} compact={{ below: 300, width: 240, height: 220 }} {script} {reset} {poster}>
		{#snippet children({ compact })}
			<div class="page absolute inset-x-6 top-4 bottom-5 flex flex-col gap-2.5 rounded-fc-lg bg-fc-page p-3.5">
				<span class="flex items-center gap-1.5 font-fc-title text-fc-sm font-bold text-fc-fg">
					<Icon icon={icons.history} size={15} class="text-fc-fg-muted" />Historique
				</span>
				{#each REVISIONS as revision, i (i)}
					{#if i === 1 || !compact}
						<div class="flex items-center justify-between gap-2 rounded-fc-md bg-fc-component py-2 pr-2 pl-3">
							<div class="flex min-w-0 flex-col">
								<span class="truncate text-fc-xs font-medium text-fc-fg">{revision.title}</span>
								<span class="truncate text-[10px] leading-3.5 text-fc-fg-muted">{revision.date}</span>
							</div>
							{#if i === 1}
								<span bind:this={target}>
									<Button variant="secondary" size="sm" class="h-7 px-2.5 text-[11px]">Restaurer</Button>
								</span>
							{:else}
								<Button variant="secondary" size="sm" class="h-7 px-2.5 text-[11px]">Restaurer</Button>
							{/if}
						</div>
					{/if}
				{/each}
			</div>
			{#if restored}
				<div
					class="toast absolute right-8 bottom-8 left-8 flex items-center gap-2 rounded-fc-md bg-fc-component px-3 py-2.5 text-fc-xs font-medium text-fc-fg"
					in:grow={{ duration: 300 }}
					out:grow={{ duration: 250 }}
				>
					<Icon icon={icons.check} size={15} class="text-fc-success" />Version restaurée
				</div>
			{/if}
			<Cursor {cursor} />
		{/snippet}
	</Stage>
</Tile>

<style>
	.page {
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.25);
	}
	.toast {
		box-shadow: 0 12px 28px -10px rgb(0 0 0 / 0.3);
	}
</style>
