<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import {
		PreferencesSection,
		ProfileSection,
		SettingsLayout,
		icons,
		readTheme,
		sectionHref,
		settingsRoute
	} from '@facile/muse';
	import { session } from '$lib/session.svelte';
	import { saveTheme } from '$lib/theme';

	const sections = [
		{ id: 'profile', label: 'Profil', icon: icons.userCircle, href: sectionHref('/settings', 'profile') },
		{ id: 'preferences', label: 'Préférences', icon: icons.palette, href: sectionHref('/settings', 'preferences') }
	];

	const route = $derived(settingsRoute(page.params.section, sections));
	const role = $derived(
		session.space ? { member: 'Membre', admin: 'Administrateur', owner: 'Propriétaire' }[session.space.role] : undefined
	);

	$effect(() => {
		if (!route.known) void goto('/settings', { replaceState: true });
	});
</script>

<svelte:head><title>Réglages · Docs</title></svelte:head>

<div class="mx-auto w-full max-w-5xl px-4 py-6 sm:px-6 md:py-10">
	<SettingsLayout {sections} active={page.params.section} backHref="/settings" title="Réglages">
		{#snippet children(current)}
			{#if current === 'profile'}
				<ProfileSection
					title="Profil"
					name={session.user?.name || session.user?.email || ''}
					email={session.user?.email}
					avatar={session.user?.avatar_url}
					{role}
				/>
			{:else}
				<PreferencesSection title="Préférences" theme={readTheme()} onThemeChange={saveTheme} />
			{/if}
		{/snippet}
	</SettingsLayout>
</div>
