<script lang="ts">
	import '../app.css';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import {
		AuthScreen,
		Drawer,
		IconButton,
		Icon,
		MobileNav,
		SideBar,
		Spinner,
		Toaster,
		Topbar,
		icons,
		type UserMenuOptions
	} from '@facile/muse';
	import { backend } from '$lib/backend';
	import { logoIcon } from '$lib/brand';
	import { rememberReturn, takeReturn } from '$lib/return';
	import { session } from '$lib/session.svelte';
	import { restoreTheme, saveTheme } from '$lib/theme';
	import { tree } from '$lib/tree.svelte';
	import Search from '$lib/components/Search.svelte';
	import Tree from '$lib/components/Tree.svelte';

	let { children } = $props();

	let collapsed = $state(false);
	let searching = $state(false);
	let treeOpen = $state(false);

	const path = $derived(page.url.pathname);
	const bare = $derived(path.startsWith('/s/'));
	const onSettings = $derived(path.startsWith('/settings'));

	restoreTheme();

	$effect(() => {
		if (!bare && !session.ready) void session.load();
	});

	$effect(() => {
		if (bare || !session.ready) return;
		if (!session.user) return rememberReturn(page.url);
		const next = takeReturn();
		if (next && next !== `${page.url.pathname}${page.url.search}`) void goto(next, { replaceState: true });
	});

	$effect(() => {
		if (session.user) void tree.load(session.spaceId).catch(() => undefined);
	});

	$effect(() => {
		if (path) treeOpen = false;
	});

	const pages = $derived([
		{ label: 'Accueil', href: '/', icon: icons.home, active: path === '/' },
		{ label: 'Corbeille', href: '/trash', icon: icons.remove, active: path.startsWith('/trash') }
	]);
	const navUser = $derived({
		name: session.user?.name || session.user?.email || 'Compte',
		email: session.user?.email,
		avatar: session.user?.avatar_url || undefined
	});
	const userMenu = $derived<UserMenuOptions>({
		settingsHref: '/settings',
		onThemeChange: saveTheme,
		onSignOut: async () => {
			await session.logout();
			await goto('/');
		}
	});
	const spaces = $derived(session.spaces.map((space) => ({ id: space.id, name: space.name })));
</script>

{#if bare}
	{@render children()}
{:else if !session.ready}
	<div class="flex h-dvh w-full items-center justify-center bg-fc-page"><Spinner /></div>
{:else if !session.user}
	<AuthScreen appName="Docs" href={backend.loginUrl()} />
{:else}
	<div class="flex h-dvh w-full overflow-hidden bg-fc-page">
		<div class="hidden h-full shrink-0 gap-3 p-3 md:flex">
			<SideBar
				icon={logoIcon}
				title="Docs"
				bind:collapsed
				{pages}
				showSearch
				onSearch={() => (searching = true)}
				{spaces}
				activeSpaceId={session.spaceId}
				onSpaceSelect={(id) => session.select(id)}
				personalSpaceLabel="Personnel"
				user={navUser}
				userHref="/settings"
				userActive={onSettings}
				{userMenu}
				class="h-full"
			/>
			<aside class="flex h-full w-64 flex-col overflow-hidden rounded-fc-lg bg-fc-component" aria-label="Documents">
				<Tree />
			</aside>
		</div>

		<main class="min-w-0 flex-1 overflow-auto overscroll-contain pb-28 md:pb-0">
			<Topbar class="md:hidden">
				<span class="flex flex-1 items-center gap-2 text-fc-md font-semibold text-fc-fg">
					<Icon icon={logoIcon} size={20} />Docs
				</span>
				<IconButton variant="ghost" aria-label="Rechercher" onclick={() => (searching = true)}>
					<Icon icon={icons.search} size={18} />
				</IconButton>
				<IconButton variant="ghost" aria-label="Documents" onclick={() => (treeOpen = true)}>
					<Icon icon={icons.menu} size={18} />
				</IconButton>
			</Topbar>
			{@render children()}
		</main>

		<MobileNav items={pages} user={navUser} profileHref="/settings" profileActive={onSettings} {userMenu} />
	</div>

	<Drawer open={treeOpen} title="Documents" onClose={() => (treeOpen = false)}>
		<Tree />
	</Drawer>
	<Search bind:open={searching} />
{/if}

<Toaster />
