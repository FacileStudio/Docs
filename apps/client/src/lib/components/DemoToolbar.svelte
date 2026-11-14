<script lang="ts">
	import { Icon, Presence, icons, type PresencePeer } from '@facile/muse';
	import { shareIcon } from '$lib/brand';

	let { compact, peers }: { compact: boolean; peers: PresencePeer[] } = $props();
</script>

<div class="dd-head">
	<Presence {peers} class="dd-presence" />
	<span class="dd-actions">
		{#if !compact}
			<span class="dd-action"><Icon icon={icons.history} size={16} />Historique</span>
			<span class="dd-action"><Icon icon={icons.download} size={16} />Exporter</span>
		{/if}
		<span class="dd-icon"><Icon icon={shareIcon} size={18} /></span>
	</span>
</div>

<style>
	.dd-head {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 44px;
	}
	.dd-head :global(.dd-presence) {
		flex: 1;
	}
	.dd-actions {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-left: auto;
		color: var(--color-fc-fg);
		font-size: 13px;
	}
	.dd-action {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 32px;
		padding: 0 10px;
		border-radius: var(--radius-fc-md);
	}
	.dd-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
	}
</style>
