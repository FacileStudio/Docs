<script lang="ts">
	import { page } from '$app/state';
	import { Button, EmptyState, icons } from '@facile/muse';

	const missing = $derived(page.status === 404);
</script>

<svelte:head><title>{missing ? 'Page introuvable' : 'Erreur'} · Docs</title></svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col px-4 py-16 sm:px-6">
	<EmptyState
		icon={missing ? icons.search : icons.warning}
		title={missing ? 'Page introuvable' : 'Une erreur est survenue'}
		description={missing
			? 'Cette adresse ne mène à aucune page de Docs.'
			: 'Rechargez la page. Si l’erreur revient, réessayez plus tard.'}
	>
		<Button href="/" icon={icons.home}>Retour à l’accueil</Button>
	</EmptyState>
</div>
