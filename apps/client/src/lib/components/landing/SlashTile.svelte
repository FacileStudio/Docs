<script lang="ts">
	import { BLOCK_ICONS, labelsFor } from '@facile/encre';
	import { Bar, Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { Icon } from '@facile/muse';
	import { formatDay, locale, t } from '$lib/i18n';

	const labels = $derived(labelsFor(locale()));
	const items = $derived([
		{ id: 'heading2', label: labels.heading(2) },
		{ id: 'bulletList', label: labels.bulletList },
		{ id: 'table', label: labels.table }
	]);
	const cells = $derived([
		[t('demo.slash.team'), t('demo.slash.date')],
		[t('demo.slash.design'), formatDay(new Date(Date.UTC(2026, 11, 12)))],
		[t('demo.slash.product'), formatDay(new Date(Date.UTC(2026, 11, 19)))]
	]);

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
		await act.wait(500);
		index = 2;
		await act.wait(900);
		open = false;
		typed = '';
		table = true;
		await act.wait(2800);
		reset();
		await act.wait(500);
	}
</script>

<Tile title={t('demo.slash.title')} body={t('demo.slash.body')}>
	<Stage width={358} height={256} compact={{ below: 300, width: 240, height: 220 }} {script} {reset} {poster}>
		{#snippet children({ compact })}
			<div class="page absolute inset-x-6 top-4 bottom-5 flex flex-col gap-2.5 rounded-fc-lg bg-fc-page p-4">
				<span class="font-fc-title text-fc-sm font-bold text-fc-fg">{t('demo.slash.page')}</span>
				<Bar w="92%" h={7} />
				{#if !compact}<Bar w="64%" h={7} />{/if}
				<div class="relative h-[80px]">
					<div class="swap absolute inset-x-0 top-0 pt-1" class:shown={table}>
						<table class="w-full text-[11px] leading-4 text-fc-fg">
							<tbody>
								{#each cells as row, r (r)}
									<tr class={r % 2 === 1 ? 'bg-fc-surface' : ''}>
										{#each row as cell, c (c)}
											<td class="px-2 py-1 {r === 0 ? 'font-semibold' : ''}">{cell}</td>
										{/each}
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
					<div class="swap absolute inset-x-0 top-0 h-4 text-fc-xs leading-4 text-fc-fg" class:shown={!table}>
						{typed}<span class="caret"></span>
					</div>
				</div>
			</div>
			<div
				class="menu swap absolute top-[120px] left-9 flex w-44 flex-col gap-0.5 rounded-fc-md bg-fc-bg p-1 text-fc-fg"
				class:shown={open}
				aria-hidden={!open}
			>
				{#each items as item, i (item.id)}
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
	.swap {
		opacity: 0;
		transform: translateY(4px) scale(0.98);
		transition:
			opacity 0.3s ease,
			transform 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	.shown {
		opacity: 1;
		transform: none;
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
