<script lang="ts">
	import { BLOCK_ICONS, labelsFor } from '@facile/encre';
	import { Bar, Stage, Tile, grow, type Act } from '@facile/mise-en-scene';
	import { Icon } from '@facile/muse';

	const LABELS = labelsFor('fr');
	const ITEMS = [
		{ id: 'taskList', label: LABELS.taskList },
		{ id: 'table', label: LABELS.table },
		{ id: 'diagram', label: LABELS.diagram }
	];
	const ROWS = [
		['Équipe', 'Date'],
		['Design', '12 déc.'],
		['Produit', '19 déc.']
	];

	let typed = $state('');
	let open = $state(false);
	let index = $state(0);
	let table = $state(false);

	function reset() {
		typed = '';
		open = false;
		index = 0;
		table = false;
	}

	function poster() {
		reset();
		table = true;
	}

	async function script(act: Act) {
		await act.wait(800);
		typed = '/';
		open = true;
		await act.wait(800);
		index = 1;
		await act.wait(900);
		open = false;
		typed = '';
		table = true;
		await act.wait(2800);
		reset();
		await act.wait(500);
	}
</script>

<Tile title="Un bloc pour chaque idée" body="Tapez / pour insérer un tableau, un diagramme ou une liste de tâches.">
	<Stage width={358} height={256} compact={{ below: 300, width: 240, height: 220 }} {script} {reset} {poster}>
		{#snippet children({ compact })}
			<div class="page absolute inset-x-6 top-4 bottom-5 flex flex-col gap-2.5 rounded-fc-lg bg-fc-page p-4">
				<span class="font-fc-title text-fc-sm font-bold text-fc-fg">Planning d’hiver</span>
				<Bar w="92%" h={7} />
				{#if !compact}<Bar w="64%" h={7} />{/if}
				{#if table}
					<div class="pt-1" in:grow={{ duration: 350 }} out:grow={{ duration: 250 }}>
						<table class="w-full text-[11px] leading-4 text-fc-fg">
							<tbody>
								{#each ROWS as row, r (r)}
									<tr class={r % 2 === 1 ? 'bg-fc-surface' : ''}>
										{#each row as cell, c (c)}
											<td class="px-2 py-1 {r === 0 ? 'font-semibold' : ''}">{cell}</td>
										{/each}
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else}
					<div class="h-4 text-fc-xs leading-4 text-fc-fg">
						{typed}<span class="caret"></span>
					</div>
				{/if}
			</div>
			{#if open}
				<div
					class="menu absolute top-[120px] left-9 flex w-44 flex-col gap-0.5 rounded-fc-md bg-fc-bg p-1 text-fc-fg"
					in:grow={{ duration: 250 }}
					out:grow={{ duration: 200 }}
				>
					{#each ITEMS as item, i (item.id)}
						<span
							class="flex h-7 items-center gap-2 rounded-fc-sm px-2 text-fc-xs transition-colors {i === index
								? 'bg-fc-surface'
								: ''}"
						>
							<Icon icon={BLOCK_ICONS[item.id]} size={14} class="text-fc-fg-muted" />
							<span class="truncate">{item.label}</span>
						</span>
					{/each}
				</div>
			{/if}
		{/snippet}
	</Stage>
</Tile>

<style>
	.page {
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.25);
	}
	.menu {
		box-shadow: 0 12px 28px -10px rgb(0 0 0 / 0.3);
	}
	.caret {
		display: inline-block;
		width: 1.5px;
		height: 1.1em;
		margin-left: 1px;
		vertical-align: text-bottom;
		background: var(--color-fc-fg);
	}
</style>
