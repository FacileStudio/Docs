<script lang="ts">
	import { page } from '$app/state';
	import { Alert, ApprovalCard, Button, EmptyState, PageHeader, Spinner, StatusDot, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { formatDate } from '$lib/format';
	import { BROWSER_ONLY, STATES, failureMessage, needsBrowserLogin, targetDocument } from '$lib/pending';
	import { rememberReturn } from '$lib/return';
	import type { PendingAction } from '$lib/types';

	const id = $derived(page.url.searchParams.get('action') ?? '');

	let action = $state<PendingAction | null | undefined>(undefined);
	let outcome = $state<'confirmed' | 'dismissed' | null>(null);
	let busy = $state(false);
	let browserOnly = $state(false);

	function failed(error: unknown, fallback: string) {
		if (needsBrowserLogin(error)) browserOnly = true;
		else toast.danger(failureMessage(error, fallback));
	}

	function signIn() {
		rememberReturn(page.url);
		window.location.assign(backend.loginUrl());
	}

	$effect(() => {
		const wanted = id;
		action = undefined;
		outcome = null;
		browserOnly = false;
		backend
			.actions()
			.then((list) => (action = list.find((item) => item.id === wanted) ?? null))
			.catch((error) => {
				action = null;
				failed(error, 'Demandes indisponibles');
			});
	});

	const documentId = $derived(action ? targetDocument(action) : null);
	const status = $derived(action ? (STATES[action.state] ?? STATES.missing) : null);
	const approval = $derived(
		action ? { id: action.id, tool: action.kind, summary: action.summary, args: parsed(action.target) } : null
	);

	function parsed(target: string): Record<string, unknown> | string {
		try {
			return JSON.parse(target) as Record<string, unknown>;
		} catch {
			return target;
		}
	}

	async function answer(confirm: boolean) {
		if (!action) return;
		busy = true;
		try {
			if (confirm) await backend.confirmAction(action.id);
			else await backend.dismissAction(action.id);
			outcome = confirm ? 'confirmed' : 'dismissed';
		} catch (error) {
			failed(error, 'Réponse impossible');
		} finally {
			busy = false;
		}
	}
</script>

<svelte:head><title>Demande d’un agent · Docs</title></svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 sm:px-6 md:py-10">
	<PageHeader title="Demande d’un agent" description="Un agent veut faire une modification qui demande votre accord." />
	{#if browserOnly}
		<Alert tone="warning" title={BROWSER_ONLY.title}>
			<p>{BROWSER_ONLY.description}</p>
			<div class="mt-3"><Button size="sm" onclick={signIn}>Se reconnecter</Button></div>
		</Alert>
	{:else if action === undefined}
		<div class="flex justify-center py-16"><Spinner /></div>
	{:else if outcome}
		<EmptyState
			icon={outcome === 'confirmed' ? icons.check : icons.close}
			title={outcome === 'confirmed' ? 'Modification appliquée' : 'Demande refusée'}
			description={outcome === 'confirmed'
				? 'La page a été remplacée par la version de l’agent.'
				: 'La page reste telle quelle.'}
		>
			{#if documentId && outcome === 'confirmed'}<Button href="/d/{documentId}">Ouvrir la page</Button>{/if}
		</EmptyState>
	{:else if !action || !approval || !status}
		<EmptyState
			icon={icons.warning}
			title="Demande introuvable"
			description="Elle a déjà été confirmée, refusée ou a expiré, ou elle n’est pas à votre nom."
		>
			<Button href="/">Retour à l’accueil</Button>
		</EmptyState>
	{:else}
		<div class="flex flex-wrap items-center gap-3 text-fc-xs text-fc-fg-muted">
			<StatusDot tone={status.tone} label={status.label} />
			<span>Client {action.client_id}</span>
			<span>Expire le {formatDate(action.expires_at)}</span>
			{#if documentId}<a class="text-fc-fg underline" href="/d/{documentId}">Voir la page actuelle</a>{/if}
		</div>
		<ApprovalCard {approval} {busy} onApprove={() => answer(true)} onDecline={() => answer(false)} />
	{/if}
</div>
