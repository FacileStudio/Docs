<script lang="ts">
	import { Bar, Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { Icon } from '@facile/muse';
	import { formatEuros, t } from '$lib/i18n';
	import { suiteIcons } from '$lib/suite-icons';

	const LEA = 'var(--color-fc-chart-1)';

	let editing = $state(false);
	let updated = $state(false);
	let pulse = $state(false);

	const rows = $derived([
		{ label: t('demo.embed.hosting'), value: 120, live: false },
		{ label: t('demo.embed.design'), value: updated ? 520 : 480, live: true },
		{ label: t('demo.embed.ads'), value: 300, live: false }
	]);
	const total = $derived(rows.reduce((sum, row) => sum + row.value, 0));

	function reset() {
		editing = false;
		updated = false;
		pulse = false;
	}

	function poster() {
		editing = true;
		updated = true;
	}

	async function script(act: Act) {
		await act.wait(900);
		editing = true;
		await act.wait(1100);
		updated = true;
		pulse = true;
		await act.wait(400);
		pulse = false;
		await act.wait(2000);
		editing = false;
		await act.wait(900);
		reset();
		await act.wait(500);
	}
</script>

<Tile title={t('demo.embed.title')} body={t('demo.embed.body')}>
	<Stage width={358} height={256} compact={{ below: 300, width: 240, height: 220 }} {script} {reset} {poster}>
		{#snippet children({ compact })}
			<div class="page absolute inset-x-6 top-4 bottom-5 flex flex-col gap-2.5 rounded-fc-lg bg-fc-page p-4">
				<span class="truncate font-fc-title text-fc-sm font-bold text-fc-fg">{t('demo.embed.page')}</span>
				<Bar w="88%" h={7} />
				{#if !compact}<Bar w="60%" h={7} />{/if}
				<div class="flex flex-col gap-1 rounded-fc-md bg-fc-component p-2">
					<span class="flex items-center gap-1.5 truncate text-[10px] leading-3 font-medium text-fc-fg-muted">
						<Icon icon={suiteIcons.feuilles} size={12} />Feuilles · {t('demo.embed.book')}
					</span>
					<table class="w-full text-[11px] leading-4 text-fc-fg tabular-nums">
						<tbody>
							{#each rows as row (row.label)}
								<tr>
									<td class="px-1.5 py-0.5">{row.label}</td>
									<td class="relative px-1.5 py-0.5 text-right">
										<span class="relative inline-block px-1">
											{#if row.live}
												<span class="cell" class:cell-on={editing} style:--lea={LEA}></span>
												<span class="label" class:label-on={editing} style:--lea={LEA}>Léa</span>
											{/if}
											<span class="relative">{formatEuros(row.value)}</span>
										</span>
									</td>
								</tr>
							{/each}
							<tr class="font-semibold">
								<td class="px-1.5 py-0.5">{t('demo.embed.total')}</td>
								<td class="px-1.5 py-0.5 text-right"
									><span class="sum" class:sum-on={pulse}>{formatEuros(total)}</span></td
								>
							</tr>
						</tbody>
					</table>
				</div>
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.page {
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.25);
	}
	.cell {
		position: absolute;
		inset: 0;
		border-radius: 3px;
		background: color-mix(in oklch, var(--lea) 22%, transparent);
		opacity: 0;
		transition: opacity 0.3s ease;
	}
	.label {
		position: absolute;
		top: 50%;
		right: calc(100% + 4px);
		padding: 0 4px;
		border-radius: 3px;
		background: var(--lea);
		color: white;
		font-size: 9px;
		font-weight: 600;
		line-height: 13px;
		opacity: 0;
		transform: translate(3px, -50%);
		transition:
			opacity 0.3s ease,
			transform 0.3s ease;
	}
	.cell-on {
		opacity: 1;
	}
	.label-on {
		opacity: 1;
		transform: translate(0, -50%);
	}
	.sum {
		display: inline-block;
		transition: transform 0.35s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	.sum-on {
		transform: scale(1.08);
	}
</style>
