<script lang="ts">
	import { tick } from 'svelte';
	import { Bar, Cursor, Stage, Window, cursorAt, grow, type Act } from '@facile/mise-en-scene';
	import { Icon, type PresencePeer } from '@facile/muse';
	import { logoIcon } from '$lib/brand';
	import DemoToolbar from './DemoToolbar.svelte';
	import DemoTree from './DemoTree.svelte';

	const LEA: PresencePeer = { id: 'lea', name: 'Léa Martin', color: 'var(--color-fc-chart-1)' };
	const LEA_LINE = 'Valider le budget avec Hugo';
	const OWN_LINE = 'Réserver la salle jeudi';

	let peers = $state<PresencePeer[]>([]);
	let lea = $state('');
	let own = $state('');
	let leaTyping = $state(false);
	let ownTyping = $state(false);
	let cursor = $state(cursorAt());
	let slot = $state<HTMLElement>();

	function reset() {
		peers = [];
		lea = '';
		own = '';
		leaTyping = false;
		ownTyping = false;
		cursor.visible = false;
	}

	function poster() {
		peers = [LEA];
		lea = LEA_LINE;
		own = OWN_LINE;
	}

	async function leaWrites(act: Act) {
		await act.wait(900);
		peers = [LEA];
		await act.wait(900);
		leaTyping = true;
		await tick();
		await act.wait(400);
		await act.type(LEA_LINE, (value) => (lea = value), 70);
	}

	async function script(act: Act) {
		await leaWrites(act);
		await act.wait(500);
		cursor.visible = true;
		await act.move(cursor, act.point(slot, 0.12, 0.5));
		await act.click(cursor, () => (ownTyping = true));
		await act.wait(250);
		cursor.visible = false;
		await act.type(OWN_LINE, (value) => (own = value), 75);
		ownTyping = false;
		await act.wait(2200);
		reset();
		await act.wait(700);
	}
</script>

{#snippet page(compact: boolean)}
	<div class="dd-page" class:dd-compact={compact}>
		<DemoToolbar {compact} {peers} />
		<div class="dd-title">
			<span class="dd-glyph"><Icon icon={logoIcon} size={compact ? 22 : 26} /></span>
			<span>Lancement d’automne</span>
		</div>
		<div class="dd-lines">
			<Bar w="96%" h={8} />
			<Bar w="88%" h={8} />
			<Bar w={compact ? '52%' : '64%'} h={8} />
		</div>
		<span class="dd-h2">Ordre du jour</span>
		<ul class="dd-list">
			<li><Bar w={compact ? '70%' : '46%'} h={8} class="dd-bar" /></li>
			<li><Bar w={compact ? '58%' : '38%'} h={8} class="dd-bar" /></li>
			{#if leaTyping || lea}
				<li transition:grow>
					<span class="dd-text"
						>{lea}{#if leaTyping}<span class="encre-caret" style:--encre-caret={LEA.color}
								><span class="encre-caret-label">Léa</span></span
							>{/if}</span
					>
				</li>
			{/if}
			{#if ownTyping || own}
				<li transition:grow>
					<span class="dd-text"
						>{own}{#if ownTyping}<span class="dd-own-caret"></span>{/if}</span
					>
				</li>
			{/if}
		</ul>
		<div class="dd-slot" bind:this={slot}></div>
		{#if !compact}
			<div class="dd-lines">
				<Bar w="92%" h={8} />
				<Bar w="74%" h={8} />
			</div>
		{/if}
	</div>
{/snippet}

{#snippet tree()}<DemoTree />{/snippet}

<Stage
	width={960}
	height={552}
	compact={{ below: 640, width: 358, height: 448 }}
	{script}
	{reset}
	{poster}
	label="Deux personnes écrivent dans le même document Docs : Léa ajoute une ligne, puis vous en ajoutez une autre."
>
	{#snippet children({ compact })}
		{#if compact}
			<Window style="top:12px;left:12px;width:334px;height:424px">
				{@render page(true)}
			</Window>
		{:else}
			<Window style="top:16px;left:40px;width:880px;height:520px" sidebar={tree}>
				{@render page(false)}
			</Window>
		{/if}
		<Cursor {cursor} />
	{/snippet}
</Stage>

<style>
	.dd-page {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-width: 0;
		padding: 18px 56px 24px;
		color: var(--color-fc-fg);
		font-family: var(--font-fc-body);
	}
	.dd-compact {
		padding: 14px 18px 18px;
	}
	.dd-title {
		display: flex;
		align-items: center;
		gap: 10px;
		margin: 18px 0 16px;
		font-family: var(--font-fc-title);
		font-size: 30px;
		font-weight: 700;
		line-height: 1.15;
	}
	.dd-compact .dd-title {
		margin: 12px 0 14px;
		font-size: 22px;
	}
	.dd-glyph {
		display: flex;
		color: var(--color-fc-fg-muted);
	}
	.dd-lines {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 4px 0;
	}
	.dd-h2 {
		margin: 22px 0 6px;
		font-family: var(--font-fc-title);
		font-size: 19px;
		font-weight: 700;
	}
	.dd-compact .dd-h2 {
		margin-top: 18px;
		font-size: 17px;
	}
	.dd-list {
		margin: 0;
		padding: 0 0 0 20px;
		list-style: disc;
	}
	.dd-list li {
		padding: 7px 0;
		font-size: 15px;
		line-height: 1.5;
	}
	.dd-list li::marker {
		color: var(--color-fc-fg-muted);
	}
	.dd-list :global(.dd-bar) {
		display: inline-block;
		vertical-align: middle;
	}
	.dd-text {
		white-space: nowrap;
	}
	.dd-own-caret {
		display: inline-block;
		width: 2px;
		height: 1.2em;
		margin-left: 1px;
		vertical-align: text-bottom;
		background: var(--color-fc-fg);
	}
	.dd-slot {
		height: 30px;
	}
</style>
