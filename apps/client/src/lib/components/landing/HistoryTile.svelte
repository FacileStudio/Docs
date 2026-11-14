<script lang="ts">
	import { Cursor, Stage, Tile, cursorAt, type Act } from '@facile/mise-en-scene';
	import { Button, Icon, icons } from '@facile/muse';
	import { t } from '$lib/i18n';

	const revisions = $derived([
		{ title: t('demo.slash.page'), date: t('demo.history.today') },
		{ title: t('demo.slash.page'), date: t('demo.history.yesterday') }
	]);

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

<Tile title={t('demo.history.title')} body={t('demo.history.body')}>
	<Stage width={358} height={256} compact={{ below: 300, width: 240, height: 220 }} {script} {reset} {poster}>
		{#snippet children({ compact })}
			<div class="page absolute inset-x-6 top-4 bottom-5 flex flex-col gap-2.5 rounded-fc-lg bg-fc-page p-3.5">
				<span class="flex items-center gap-1.5 font-fc-title text-fc-sm font-bold text-fc-fg">
					<Icon icon={icons.history} size={15} class="text-fc-fg-muted" />{t('demo.history')}
				</span>
				{#each revisions as revision, i (i)}
					{#if i === 1 || !compact}
						<div class="flex items-center justify-between gap-2 rounded-fc-md bg-fc-component py-2 pr-2 pl-3">
							<div class="flex min-w-0 flex-col">
								<span class="truncate text-fc-xs font-medium text-fc-fg">{revision.title}</span>
								<span class="truncate text-[10px] leading-3.5 text-fc-fg-muted">{revision.date}</span>
							</div>
							{#if i === 1}
								<span bind:this={target}>
									<Button variant="secondary" size="sm" class="h-7 px-2.5 text-[11px]"
										>{t('demo.history.restore')}</Button
									>
								</span>
							{:else}
								<Button variant="secondary" size="sm" class="h-7 px-2.5 text-[11px]">{t('demo.history.restore')}</Button
								>
							{/if}
						</div>
					{/if}
				{/each}
			</div>
			<div
				class="toast absolute right-8 bottom-8 left-8 flex items-center gap-2 rounded-fc-md bg-fc-component px-3 py-2.5 text-fc-xs font-medium text-fc-fg"
				class:shown={restored}
				aria-hidden={!restored}
			>
				<Icon icon={icons.check} size={15} class="text-fc-success" />{t('demo.history.restored')}
			</div>
			<Cursor {cursor} />
		{/snippet}
	</Stage>
</Tile>

<style>
	.page {
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.25);
	}
	.toast {
		opacity: 0;
		transform: translateY(8px);
		transition:
			opacity 0.3s ease,
			transform 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
		box-shadow: 0 12px 28px -10px rgb(0 0 0 / 0.3);
	}
	.shown {
		opacity: 1;
		transform: none;
	}
</style>
