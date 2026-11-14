<script lang="ts">
	import { Icon, LandingFooter, LandingHeader, SignInWithFacile, icons } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { logoIcon } from '$lib/brand';
	import DocsDemo from './DocsDemo.svelte';

	const signInHref = backend.loginUrl();

	const features = [
		{
			icon: icons.usersGroup,
			title: 'Chacun voit l’autre écrire',
			body: 'Les curseurs s’affichent en direct. Rien à enregistrer, rien à fusionner.'
		},
		{
			icon: icons.layers,
			title: 'Des pages rangées en arbre',
			body: 'Chaque espace a son arbre. Glissez une page pour la déplacer.'
		},
		{
			icon: icons.history,
			title: 'Un historique par page',
			body: 'Revenez à une version passée, exportez en PDF ou en Markdown.'
		},
		{
			icon: icons.plug,
			title: 'Ouvert à vos agents',
			body: 'Le serveur MCP de Docs laisse vos agents lire et modifier les pages.'
		}
	];
</script>

<svelte:head><title>Docs · Des documents écrits à plusieurs</title></svelte:head>

<div class="min-h-dvh overflow-x-clip bg-fc-page text-fc-fg">
	<LandingHeader appName="Docs" icon={logoIcon} skipHref="#main">
		{#snippet actions()}
			<SignInWithFacile href={signInHref} data-sveltekit-reload />
		{/snippet}
	</LandingHeader>

	<main id="main">
		<section class="mx-auto flex max-w-5xl flex-col items-center px-4 pt-16 pb-12 text-center sm:px-6 sm:pt-24">
			<h1 class="font-fc-title text-fc-3xl leading-tight font-bold tracking-tight sm:text-5xl">
				Des documents écrits à plusieurs
			</h1>
			<p class="mt-5 max-w-lg text-fc-lg text-fc-fg-muted">Votre équipe écrit sur la même page, au même moment.</p>
			<div class="mt-8 flex justify-center">
				<SignInWithFacile href={signInHref} size="lg" data-sveltekit-reload />
			</div>
			<div class="mt-12 w-full sm:mt-16"><DocsDemo /></div>
		</section>

		<section class="mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-20">
			<h2 class="text-center font-fc-title text-fc-2xl font-bold tracking-tight sm:text-fc-3xl">
				Tout ce qu’il faut pour écrire ensemble
			</h2>
			<div class="mt-10 grid gap-4 sm:grid-cols-2">
				{#each features as feature (feature.title)}
					<div class="rounded-fc-lg bg-fc-component p-6">
						<span class="flex size-10 items-center justify-center rounded-fc-md bg-fc-surface">
							<Icon icon={feature.icon} size={20} />
						</span>
						<h3 class="mt-4 text-fc-md font-bold">{feature.title}</h3>
						<p class="mt-1.5 text-fc-sm text-fc-fg-muted">{feature.body}</p>
					</div>
				{/each}
			</div>
		</section>

		<section class="mx-auto max-w-5xl px-4 py-16 text-center sm:px-6 sm:py-24">
			<h2 class="font-fc-title text-fc-2xl font-bold tracking-tight sm:text-fc-3xl">Ouvrez votre première page</h2>
			<p class="mt-3 text-fc-fg-muted">Un compte Facile suffit.</p>
			<div class="mt-8 flex justify-center">
				<SignInWithFacile href={signInHref} size="lg" data-sveltekit-reload />
			</div>
		</section>
	</main>

	<LandingFooter
		appName="Docs"
		icon={logoIcon}
		tagline="Les documents collaboratifs de la suite Facile."
		legal={{ hideLocaleSwitch: true }}
	/>
</div>
