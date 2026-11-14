<script lang="ts">
	import { tick } from 'svelte';
	import { Bar, Cursor, Stage, Window, cursorAt, type Act } from '@facile/mise-en-scene';
	import { Icon, type PresencePeer } from '@facile/muse';
	import { logoIcon } from '$lib/brand';
	import { t } from '$lib/i18n';
	import DemoToolbar from './DemoToolbar.svelte';
	import DemoTree from './DemoTree.svelte';

	const LEA: PresencePeer = { id: 'lea', name: 'Léa Martin', color: 'var(--color-fc-chart-1)' };
	const HUGO: PresencePeer = { id: 'hugo', name: 'Hugo Bernard', color: 'var(--color-fc-chart-2)' };

	let peers = $state<PresencePeer[]>([]);
	let heading = $state<string | null>(null);
	let hugoSel = $state(false);
	let hugoTyping = $state(false);
	let lea = $state('');
	let own = $state('');
	let leaTyping = $state(false);
	let ownTyping = $state(false);
	let cursor = $state(cursorAt());
	let slot = $state<HTMLElement>();

	function start(text: string): string {
		return text.split(' ').slice(0, -1).join(' ');
	}

	function reset() {
		peers = [];
		heading = null;
		hugoSel = false;
		hugoTyping = false;
		lea = '';
		own = '';
		leaTyping = false;
		ownTyping = false;
		cursor.visible = false;
	}

	function poster() {
		reset();
		peers = [LEA, HUGO];
		heading = start(t('demo.heading.hugo'));
		hugoTyping = true;
		leaTyping = true;
		lea = start(t('demo.line.lea'));
		ownTyping = true;
		own = start(t('demo.line.own'));
	}

	async function hugoEdits(act: Act) {
		await act.wait(500);
		hugoTyping = true;
		await act.wait(700);
		hugoSel = true;
		await act.wait(800);
		hugoSel = false;
		await act.type(t('demo.heading.hugo'), (value) => (heading = value), 110);
	}

	async function leaWrites(act: Act) {
		leaTyping = true;
		await tick();
		await act.wait(900);
		await act.type(t('demo.line.lea'), (value) => (lea = value), 100);
	}

	async function iWrite(act: Act) {
		await act.wait(700);
		cursor.visible = true;
		await act.move(cursor, act.point(slot, 0.12, 0.5));
		await act.click(cursor, () => (ownTyping = true));
		await act.wait(200);
		cursor.visible = false;
		await act.type(t('demo.line.own'), (value) => (own = value), 105);
	}

	async function script(act: Act) {
		await act.wait(600);
		peers = [LEA, HUGO];
		await act.wait(500);
		await Promise.all([hugoEdits(act), leaWrites(act), iWrite(act)]);
		await act.wait(2200);
		reset();
		await act.wait(600);
	}
</script>

{#snippet caret(peer: PresencePeer, name: string)}<span class="encre-caret" style:--encre-caret={peer.color}
		><span class="encre-caret-label">{name}</span></span
	>{/snippet}

{#snippet page(compact: boolean)}
	<div class="dd-page" class:dd-compact={compact}>
		<DemoToolbar {compact} {peers} />
		<div class="dd-title">
			<span class="dd-glyph"><Icon icon={logoIcon} size={compact ? 22 : 26} /></span>
			<span>{t('demo.page')}</span>
		</div>
		<div class="dd-lines">
			<Bar w="96%" h={8} />
			<Bar w="88%" h={8} />
			<Bar w={compact ? '52%' : '64%'} h={8} />
		</div>
		<span class="dd-h2"
			><span class:encre-caret-selection={hugoSel} style:--encre-caret={HUGO.color}>{heading ?? t('demo.heading')}</span
			>{#if hugoTyping}{@render caret(HUGO, 'Hugo')}{/if}</span
		>
		<ul class="dd-list">
			<li><Bar w={compact ? '70%' : '46%'} h={8} class="dd-bar" /></li>
			<li><Bar w={compact ? '58%' : '38%'} h={8} class="dd-bar" /></li>
			<li class="dd-new" class:dd-shown={leaTyping || lea}>
				<span class="dd-text"
					>{lea}{#if leaTyping}{@render caret(LEA, 'Léa')}{/if}</span
				>
			</li>
			<li class="dd-new" class:dd-shown={ownTyping || own} bind:this={slot}>
				<span class="dd-text"
					>{own}{#if ownTyping}<span class="dd-own-caret"></span>{/if}</span
				>
			</li>
		</ul>
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
	label={t('demo.label')}
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
	.dd-new {
		height: 37px;
		opacity: 0;
		transform: translateY(4px);
		transition:
			opacity 0.3s ease,
			transform 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	.dd-shown {
		opacity: 1;
		transform: none;
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
</style>
