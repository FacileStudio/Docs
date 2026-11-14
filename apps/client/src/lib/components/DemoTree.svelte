<script lang="ts">
	import { Bar } from '@facile/mise-en-scene';
	import { Icon } from '@facile/muse';
	import { documentIcon } from '$lib/brand';

	const PAGES = [72, 96, 64, 84];
</script>

<div class="dd-tree">
	<span class="dd-space">Équipe</span>
	{#each PAGES as width, i (i)}
		<span class="dd-row">
			<Icon icon={documentIcon} size={16} />
			<Bar w={width} h={7} />
		</span>
	{/each}
	<span class="dd-row dd-active">
		<Icon icon={documentIcon} size={16} />
		<span class="dd-row-title">Lancement d’automne</span>
	</span>
	<span class="dd-row dd-child">
		<Icon icon={documentIcon} size={16} />
		<Bar w={70} h={7} />
	</span>
</div>

<style>
	.dd-tree {
		display: flex;
		flex-direction: column;
		gap: 2px;
		width: 220px;
		padding: 16px 12px;
		color: var(--color-fc-fg-muted);
		font-family: var(--font-fc-body);
	}
	.dd-space {
		padding: 0 8px 8px;
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0.04em;
		text-transform: uppercase;
	}
	.dd-row {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 32px;
		padding: 0 8px;
		border-radius: var(--radius-fc-md);
	}
	.dd-child {
		padding-left: 28px;
	}
	.dd-active {
		background: color-mix(in oklch, var(--color-fc-fg) 7%, transparent);
		color: var(--color-fc-fg);
	}
	.dd-row-title {
		overflow: hidden;
		font-size: 13px;
		font-weight: 500;
		white-space: nowrap;
		text-overflow: ellipsis;
	}
</style>
