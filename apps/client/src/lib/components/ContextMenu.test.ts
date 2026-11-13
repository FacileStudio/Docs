import { flushSync, mount, tick, unmount } from 'svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import ContextMenu, { type ContextMenuItem } from './ContextMenu.svelte';

let app: ReturnType<typeof mount> | null = null;

afterEach(() => {
	if (app) unmount(app);
	app = null;
	document.body.innerHTML = '';
});

function renderMenu(props: { x: number; y: number; items: ContextMenuItem[]; onclose: () => void }) {
	const target = document.createElement('div');
	document.body.append(target);
	app = mount(ContextMenu, { target, props });
	flushSync();
	return target;
}

describe('ContextMenu', () => {
	it('renders actions and separators', () => {
		const items: ContextMenuItem[] = [
			{ label: 'Ouvrir', href: '/d/test-1' },
			{ separator: true },
			{ label: 'Supprimer', tone: 'danger' }
		];
		const onclose = vi.fn();
		const target = renderMenu({ x: 100, y: 150, items, onclose });

		const menu = target.querySelector('[role="menu"]');
		expect(menu).not.toBeNull();
		const links = target.querySelectorAll('a[role="menuitem"]');
		expect(links).toHaveLength(1);
		expect(links[0].getAttribute('href')).toBe('/d/test-1');
		expect(links[0].textContent).toContain('Ouvrir');

		const separators = target.querySelectorAll('div[role="separator"]');
		expect(separators).toHaveLength(1);

		const buttons = target.querySelectorAll('button[role="menuitem"]');
		expect(buttons).toHaveLength(1);
		expect(buttons[0].textContent).toContain('Supprimer');
	});

	it('triggers action onselect and onclose when item is clicked', async () => {
		const onselect = vi.fn();
		const onclose = vi.fn();
		const items: ContextMenuItem[] = [{ label: 'Action test', onselect }];
		const target = renderMenu({ x: 50, y: 50, items, onclose });

		const button = target.querySelector<HTMLButtonElement>('button[role="menuitem"]');
		button?.click();
		await tick();

		expect(onselect).toHaveBeenCalledOnce();
		expect(onclose).toHaveBeenCalledOnce();
	});

	it('closes on Escape key press', async () => {
		const onclose = vi.fn();
		const items: ContextMenuItem[] = [{ label: 'Item 1' }];
		const target = renderMenu({ x: 50, y: 50, items, onclose });

		const menu = target.querySelector<HTMLElement>('[role="menu"]');
		menu?.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
		await tick();

		expect(onclose).toHaveBeenCalledOnce();
	});

	it('navigates with ArrowDown and ArrowUp', async () => {
		const onclose = vi.fn();
		const items: ContextMenuItem[] = [{ label: 'First' }, { label: 'Second' }, { label: 'Third' }];
		const target = renderMenu({ x: 50, y: 50, items, onclose });
		const focusables = Array.from(target.querySelectorAll<HTMLElement>('[role="menuitem"]'));

		focusables[0].focus();
		expect(document.activeElement).toBe(focusables[0]);

		const menu = target.querySelector<HTMLElement>('[role="menu"]');
		menu?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }));
		expect(document.activeElement).toBe(focusables[1]);

		menu?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }));
		expect(document.activeElement).toBe(focusables[2]);

		menu?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }));
		expect(document.activeElement).toBe(focusables[0]);

		menu?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }));
		expect(document.activeElement).toBe(focusables[2]);
	});

	it('closes when pointerdown happens outside', async () => {
		const onclose = vi.fn();
		const items: ContextMenuItem[] = [{ label: 'Inside' }];
		renderMenu({ x: 100, y: 100, items, onclose });

		const outside = document.createElement('div');
		document.body.append(outside);
		outside.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }));
		await tick();

		expect(onclose).toHaveBeenCalledOnce();
	});

	it('opens a submenu in place and comes back from it', async () => {
		const pick = vi.fn();
		const onclose = vi.fn();
		const items: ContextMenuItem[] = [{ label: 'Transformer en', items: [{ label: 'Titre 1', onselect: pick }] }];
		const target = renderMenu({ x: 0, y: 0, items, onclose });
		const entries = () => Array.from(target.querySelectorAll<HTMLButtonElement>('[role="menuitem"]'));
		entries()[0].click();
		flushSync();
		expect(entries().map((entry) => entry.textContent?.trim())).toEqual(['‹ Transformer en', 'Titre 1']);
		entries()[0].click();
		flushSync();
		expect(entries()[0].textContent).toContain('Transformer en');
		entries()[0].click();
		flushSync();
		entries()[1].click();
		expect(pick).toHaveBeenCalledOnce();
		expect(onclose).toHaveBeenCalledOnce();
		await tick();
	});
});
