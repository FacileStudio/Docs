<script module lang="ts">
	export type ContextMenuAction = {
		label: string;
		icon?: string;
		/** Draws this assistant's Bulle blob in place of an icon. */
		blob?: string;
		href?: string;
		tone?: 'default' | 'danger';
		disabled?: boolean;
		onselect?: () => void;
		/** Opens these entries in place of the menu, with a way back. */
		items?: ContextMenuItem[];
	};

	export type ContextMenuSeparator = { separator: true };

	export type ContextMenuItem = ContextMenuAction | ContextMenuSeparator;

	export interface ContextMenuProps {
		x: number;
		y: number;
		items: ContextMenuItem[];
		/** Opened from the keyboard: the first entry takes focus and shows its ring. */
		keyboard?: boolean;
		onclose: () => void;
	}
</script>

<script lang="ts">
	import { Blob } from '@facile/blob';
	import { Icon, icons } from '@facile/muse';
	import { untrack } from 'svelte';
	import { step } from '$lib/menu-keys';

	let { x, y, items, keyboard = false, onclose }: ContextMenuProps = $props();

	let keys = $state(untrack(() => keyboard));
	const opener = typeof document !== 'undefined' ? (document.activeElement as HTMLElement | null) : null;

	let submenu = $state<ContextMenuAction | null>(null);
	const back = $derived<ContextMenuAction | null>(submenu ? { label: submenu.label, icon: icons.chevronLeft } : null);
	const shown = $derived<ContextMenuItem[]>(submenu && back ? [back, ...(submenu.items ?? [])] : items);

	let menuEl = $state<HTMLElement | null>(null);
	let measured = $state<{ width: number; height: number } | null>(null);
	let winWidth = $state(typeof window !== 'undefined' ? window.innerWidth : 0);
	let winHeight = $state(typeof window !== 'undefined' ? window.innerHeight : 0);

	const left = $derived.by(() => {
		if (!measured || winWidth <= 0) return x;
		const pad = 8;
		const maxX = winWidth - measured.width - pad;
		return Math.max(pad, Math.min(x, maxX));
	});

	const top = $derived.by(() => {
		if (!measured || winHeight <= 0) return y;
		const pad = 8;
		const maxY = winHeight - measured.height - pad;
		return Math.max(pad, Math.min(y, maxY));
	});

	function measure() {
		if (!menuEl) return;
		const rect = menuEl.getBoundingClientRect();
		measured = { width: rect.width, height: rect.height };
	}

	function getFocusable(): HTMLElement[] {
		if (!menuEl) return [];
		return Array.from(menuEl.querySelectorAll<HTMLElement>('[role="menuitem"]:not([disabled])'));
	}

	function choose(action: ContextMenuAction) {
		if (action.disabled) return;
		if (action === back || action.items) {
			submenu = action === back ? null : action;
			return;
		}
		action.onselect?.();
		onclose();
	}

	function dismiss() {
		onclose();
		opener?.focus({ preventScroll: true });
	}

	function onkeydown(event: KeyboardEvent) {
		keys = true;
		if (event.key === 'Escape') {
			event.preventDefault();
			event.stopPropagation();
			dismiss();
			return;
		}
		const focusable = getFocusable();
		const current = document.activeElement as HTMLElement | null;
		if (event.key === ' ' && current?.tagName === 'A') {
			event.preventDefault();
			current.click();
			return;
		}
		const next = step(event.key, current ? focusable.indexOf(current) : -1, focusable.length);
		if (next === null || focusable.length === 0) return;
		event.preventDefault();
		focusable[next].focus({ preventScroll: true });
		focusable[next].scrollIntoView?.({ block: 'nearest' });
	}

	$effect(() => {
		measure();
	});

	$effect(() => {
		void shown;
		measure();
		const first = getFocusable()[0];
		if (untrack(() => keys) && first) first.focus();
		else menuEl?.focus();
	});

	$effect(() => {
		function onPointerDown(event: PointerEvent) {
			const target = event.target;
			if (menuEl && (target === window || !(target instanceof Node) || !menuEl.contains(target))) {
				onclose();
			}
		}
		function onWindowScroll(event: Event) {
			if (menuEl && event.target instanceof Node && menuEl.contains(event.target)) return;
			onclose();
		}
		function onWindowResize() {
			winWidth = window.innerWidth;
			winHeight = window.innerHeight;
			measure();
		}

		window.addEventListener('pointerdown', onPointerDown, true);
		window.addEventListener('scroll', onWindowScroll, { capture: true, passive: true });
		window.addEventListener('resize', onWindowResize);

		return () => {
			window.removeEventListener('pointerdown', onPointerDown, true);
			window.removeEventListener('scroll', onWindowScroll, true);
			window.removeEventListener('resize', onWindowResize);
		};
	});
</script>

<div
	bind:this={menuEl}
	role="menu"
	tabindex="-1"
	class="fixed z-50 max-h-[min(28rem,calc(100dvh-1rem))] min-w-48 overflow-y-auto overscroll-contain rounded-fc-md bg-fc-component p-1 text-fc-fg shadow-lg outline-none"
	style:left="{left}px"
	style:top="{top}px"
	{onkeydown}
	onpointerdown={() => (keys = false)}
	oncontextmenu={(event) => event.preventDefault()}
>
	{#each shown as item, i (`${submenu?.label ?? ''}:${i}`)}
		{#if 'separator' in item}
			<div role="separator" class="my-1 h-px bg-fc-surface"></div>
		{:else}
			{@const tone = item.tone === 'danger' ? 'text-fc-danger' : 'text-fc-fg'}
			{#if item.href && !item.disabled}
				<a
					role="menuitem"
					href={item.href}
					class="flex min-h-8 w-full cursor-pointer items-center gap-2 rounded-fc-sm px-2.5 py-1.5 text-left text-fc-sm transition-colors hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring {tone}"
					onclick={() => choose(item)}
				>
					{#if item.icon}<Icon icon={item.icon} size={15} />{/if}
					<span class="min-w-0 flex-1 truncate">{item.label}</span>
				</a>
			{:else}
				<button
					type="button"
					role="menuitem"
					disabled={item.disabled}
					class="flex min-h-8 w-full cursor-pointer items-center gap-2 rounded-fc-sm px-2.5 py-1.5 text-left text-fc-sm transition-colors hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring disabled:cursor-not-allowed disabled:opacity-50 {tone}"
					onclick={() => choose(item)}
				>
					{#if item.blob}<Blob name={item.blob} size={16} />{:else if item.icon}<Icon icon={item.icon} size={15} />{/if}
					<span class="min-w-0 flex-1 truncate">{item.label}</span>
					{#if item.items}<span aria-hidden="true" class="text-fc-fg-muted">›</span>{/if}
				</button>
			{/if}
		{/if}
	{/each}
</div>
