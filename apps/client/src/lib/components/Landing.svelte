<script lang="ts">
	import {
		FeatureSplit,
		ForDevelopers,
		LANDING_HEADING,
		LandingCta,
		LandingFooter,
		LandingHeader,
		PricingPlans,
		SignInWithFacile,
		WorksWith,
		icons,
		signUpUrl,
		type DeveloperRow,
		type FeatureItem,
		type WorksWithApp
	} from '@facile/muse';
	import { backend } from '$lib/backend';
	import { logoIcon, shareIcon } from '$lib/brand';
	import { locale, rememberLocale, t } from '$lib/i18n';
	import { suiteIcons } from '$lib/suite-icons';
	import DocsDemo from './DocsDemo.svelte';
	import EmbedTile from './landing/EmbedTile.svelte';
	import HistoryTile from './landing/HistoryTile.svelte';
	import SlashTile from './landing/SlashTile.svelte';

	const signInHref = backend.loginUrl();
	const FEATURES = [
		['live', icons.usersGroup],
		['tree', icons.layers],
		['share', shareIcon],
		['export', icons.download],
		['import', icons.upload],
		['trash', icons.remove]
	] as const;

	const features = $derived<FeatureItem[]>(
		FEATURES.map(([key, icon]) => ({
			icon,
			title: t(`landing.feature.${key}.title`),
			text: t(`landing.feature.${key}.text`)
		}))
	);

	const apps = $derived<WorksWithApp[]>([
		{
			name: 'Feuilles',
			icon: suiteIcons.feuilles,
			line: t('landing.with.feuilles'),
			href: 'https://feuilles.facile.studio'
		},
		{ name: 'Toile', icon: suiteIcons.toile, line: t('landing.with.toile'), href: 'https://toile.facile.studio' },
		{ name: 'Vision', icon: suiteIcons.vision, line: t('landing.with.vision'), href: 'https://vision.facile.studio' },
		{ name: 'Bulle', icon: suiteIcons.bulle, line: t('landing.with.bulle'), href: 'https://bulle.facile.studio' }
	]);

	const rows = $derived<DeveloperRow[]>([
		{ kind: 'mcp', text: t('landing.dev.mcp') },
		{ kind: 'api', text: t('landing.dev.api'), href: 'https://docs.facile.studio/docs' },
		{ kind: 'source', text: t('landing.dev.source'), href: 'https://github.com/FacileStudio/Docs' }
	]);

	const links = $derived([
		{ href: '#features', label: t('landing.nav.features') },
		{ href: '#pricing', label: t('landing.nav.pricing') }
	]);
	const columns = $derived([
		{ title: 'Docs', links },
		{
			title: 'Facile',
			links: [
				{ href: 'https://suite.facile.studio', label: t('landing.nav.suite'), external: true },
				{ href: signUpUrl(locale()), label: t('landing.nav.signup'), external: true }
			]
		}
	]);

	$effect(() => {
		document.querySelector('meta[name="description"]')?.setAttribute('content', t('landing.meta'));
	});

	const width = 'mx-auto w-full max-w-5xl px-4 sm:px-6';
</script>

<svelte:head><title>{t('landing.title')}</title></svelte:head>

<div class="min-h-dvh overflow-x-clip bg-fc-page text-fc-fg" data-sveltekit-reload>
	<LandingHeader appName="Docs" icon={logoIcon} {links} skipHref="#main">
		{#snippet actions()}
			<SignInWithFacile href={signInHref} size="sm" />
		{/snippet}
	</LandingHeader>

	<main id="main" class="flex flex-col gap-20 pb-24 md:gap-28">
		<section class="{width} flex flex-col items-center pt-16 text-center sm:pt-24">
			<h1 class="max-w-3xl font-fc-title text-fc-3xl leading-tight font-bold tracking-tight text-balance sm:text-5xl">
				{t('landing.slogan')}
			</h1>
			<p class="mt-5 max-w-2xl text-fc-lg text-balance text-fc-fg-muted">{t('landing.subtitle')}</p>
			<LandingCta {signInHref} align="center" class="mt-8 w-full" />
			<div class="mt-12 w-full sm:mt-16"><DocsDemo /></div>
		</section>

		<section aria-labelledby="tiles-title" class={width}>
			<h2 id="tiles-title" class="mb-10 text-center {LANDING_HEADING}">{t('landing.tiles')}</h2>
			<div class="grid gap-6 md:grid-cols-3">
				<SlashTile />
				<HistoryTile />
				<EmbedTile />
			</div>
		</section>

		<FeatureSplit
			id="features"
			class="{width} scroll-mt-24"
			title={t('landing.features')}
			description={t('landing.featuresLine')}
			{features}
		/>

		<WorksWith class={width} {apps} description={t('landing.with.line')} />

		<section class="{width} flex flex-col items-center gap-5 text-center">
			<h2 class={LANDING_HEADING}>{t('landing.cta.title')}</h2>
			<p class="text-fc-md text-fc-fg-muted">{t('landing.cta.line')}</p>
			<LandingCta {signInHref} align="center" class="w-full" />
			<p class="text-fc-sm text-fc-fg-muted">{t('landing.trust')}</p>
		</section>

		<PricingPlans
			id="pricing"
			class="{width} scroll-mt-24"
			title={t('landing.nav.pricing')}
			headingClass={LANDING_HEADING}
			highlight={['ai.access']}
			highlightOnly
			notes={{ '*': t('landing.plan.paid'), solo: t('landing.plan.included') }}
			recommended="pro"
			suiteHref="https://suite.facile.studio/#tarifs"
		/>

		<div class={width}>
			<ForDevelopers
				{rows}
				description={t('landing.dev.line')}
				selfHostHref="https://suite.facile.studio/auto-hebergement"
			/>
		</div>
	</main>

	<LandingFooter
		appName="Docs"
		icon={logoIcon}
		tagline={t('landing.line')}
		{columns}
		legal={{ onLocaleChange: rememberLocale }}
	/>
</div>
