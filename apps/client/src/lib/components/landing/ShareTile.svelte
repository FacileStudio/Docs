<script lang="ts">
	import { Cursor, Stage, Tile, cursorAt, grow, type Act } from '@facile/mise-en-scene';
	import { Button, Icon, Switch, icons } from '@facile/muse';

	let nested = $state(false);
	let link = $state(false);
	let cursor = $state(cursorAt());
	let toggle = $state<HTMLElement>();
	let create = $state<HTMLElement>();

	function reset() {
		nested = false;
		link = false;
		cursor.visible = false;
	}

	function poster() {
		reset();
		nested = true;
		link = true;
	}

	async function script(act: Act) {
		await act.wait(900);
		cursor.visible = true;
		await act.move(cursor, act.point(toggle, 0.6, 0.6));
		await act.click(cursor, () => (nested = true));
		await act.wait(450);
		await act.move(cursor, act.point(create, 0.6, 0.65));
		await act.click(cursor, () => (link = true));
		await act.wait(300);
		cursor.visible = false;
		await act.wait(2600);
		reset();
		await act.wait(600);
	}
</script>

<Tile
	title="Un lien pour lire sans compte"
	body="Partagez une page, et ses sous-pages si vous voulez, en lecture seule."
>
	<Stage width={358} height={256} compact={{ below: 300, width: 240, height: 220 }} {script} {reset} {poster}>
		{#snippet children()}
			<div class="page absolute inset-x-6 top-4 bottom-5 flex flex-col gap-3 rounded-fc-lg bg-fc-page p-4">
				<span class="font-fc-title text-fc-sm font-bold text-fc-fg">Partager</span>
				{#if link}
					<div class="flex flex-col gap-2" in:grow={{ duration: 350 }} out:grow={{ duration: 250 }}>
						<span class="text-[11px] leading-4 text-fc-fg-muted">Toute personne qui a ce lien peut lire la page.</span>
						<span class="flex h-8 items-center gap-2 rounded-fc-md bg-fc-surface pr-2 pl-3 text-fc-fg">
							<span class="min-w-0 flex-1 truncate font-fc-mono text-[11px]">docs.facile.studio/s/k7Q2vXa9</span>
							<Icon icon={icons.copy} size={14} class="text-fc-fg-muted" />
						</span>
					</div>
				{:else}
					<div class="flex flex-col gap-3" in:grow={{ duration: 350 }} out:grow={{ duration: 250 }}>
						<span class="text-[11px] leading-4 text-fc-fg-muted"
							>Un lien public donne accès en lecture seule, sans compte Facile.</span
						>
						<div class="flex items-center justify-between gap-3">
							<span class="text-fc-xs font-medium text-fc-fg">Inclure les sous-pages</span>
							<span bind:this={toggle} class="flex"
								><Switch checked={nested} aria-label="Inclure les sous-pages" /></span
							>
						</div>
					</div>
				{/if}
				<div class="mt-auto flex justify-end">
					{#if link}
						<Button size="sm" class="h-7 px-3 text-[11px]">Terminé</Button>
					{:else}
						<span bind:this={create} class="flex"
							><Button size="sm" class="h-7 px-3 text-[11px]">Créer le lien</Button></span
						>
					{/if}
				</div>
			</div>
			<Cursor {cursor} />
		{/snippet}
	</Stage>
</Tile>

<style>
	.page {
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.25);
	}
</style>
