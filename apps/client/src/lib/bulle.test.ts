import { afterEach, describe, expect, it, vi } from 'vitest';
import { backend } from './backend';
import { assistant, bulleMessage, embedUrl, rememberSidebar, sidebarOpen, trustedOrigin } from './bulle';
import { editorMenu, type MenuEditor } from './editor-menu';

afterEach(() => vi.restoreAllMocks());

describe('bulle', () => {
	it('builds the chat embed URL of a page', () => {
		const url = new URL(
			embedUrl({ app: 'docs', id: 'd1', title: 'Plan & co', url: 'https://docs.facile.studio/d/d1' })
		);
		expect(url.origin + url.pathname).toBe('https://bulle.facile.studio/embed/chat');
		expect(Object.fromEntries(url.searchParams)).toEqual({
			app: 'docs',
			id: 'd1',
			title: 'Plan & co',
			url: 'https://docs.facile.studio/d/d1'
		});
	});

	it('trusts Bulle, and localhost only while developing', () => {
		expect(trustedOrigin('https://bulle.facile.studio')).toBe(true);
		expect(trustedOrigin('https://evil.example')).toBe(false);
		expect(trustedOrigin('http://localhost:5173')).toBe(false);
		expect(trustedOrigin('http://localhost:5173', true)).toBe(true);
	});

	it('reads only the two messages Bulle sends', () => {
		expect(bulleMessage({ type: 'bulle:close' })).toEqual({ type: 'bulle:close' });
		expect(bulleMessage({ type: 'bulle:ready', assistant: { name: 'Nova' } })?.type).toBe('bulle:ready');
		expect(bulleMessage({ type: 'bulle:ready' })).toBeNull();
		expect(bulleMessage('bulle:close')).toBeNull();
	});

	it('remembers the sidebar per person', () => {
		rememberSidebar('a@x', true);
		expect([sidebarOpen('a@x'), sidebarOpen('b@x')]).toEqual([true, false]);
	});

	it('asks the assistant once and falls back to Bulle, asking again after a failure', async () => {
		const ask = vi.spyOn(backend, 'assistant').mockRejectedValueOnce(new Error('down'));
		expect((await assistant()).name).toBe('Bulle');
		ask.mockResolvedValue({ name: 'Nova', accent: null });
		expect((await assistant()).name).toBe('Nova');
		expect((await assistant()).name).toBe('Nova');
		expect(ask).toHaveBeenCalledTimes(2);
	});
});

describe('editorMenu', () => {
	const editor: MenuEditor = {
		instance: () => null,
		turnInto: vi.fn(() => true),
		duplicateBlock: vi.fn(() => true),
		deleteSelection: vi.fn(() => true),
		copyBlockLink: vi.fn(() => 'b1')
	};
	const base = {
		editor,
		empty: false,
		markdown: 'Bonjour',
		pageUrl: 'https://docs.facile.studio/d/d1',
		assistant: 'Nova',
		onask: vi.fn(),
		onpaste: vi.fn(),
		oncopy: vi.fn(),
		oncopied: vi.fn()
	};
	const labels = (items: ReturnType<typeof editorMenu>) =>
		items.map((item) => ('separator' in item ? '—' : `${item.label}${item.disabled ? ' (off)' : ''}`));

	it('offers the clipboard, block actions and the assistant', () => {
		const items = editorMenu({ ...base, editable: true });
		expect(labels(items)).toEqual([
			'Couper',
			'Copier',
			'Coller',
			'—',
			'Transformer en',
			'Dupliquer',
			'Supprimer',
			'Copier le lien du bloc',
			'—',
			'Demander à Nova'
		]);
		const ask = items.at(-1);
		if (ask && !('separator' in ask)) ask.onselect?.();
		expect(ask && !('separator' in ask) && ask.blob).toBe('Nova');
		expect(base.onask).toHaveBeenCalledWith('Bonjour');
		const link = items[7];
		if (!('separator' in link)) link.onselect?.();
		expect(editor.copyBlockLink).toHaveBeenCalledWith('https://docs.facile.studio/d/d1');
	});

	it('disables writes on a read-only page', () => {
		const items = editorMenu({ ...base, editable: false, empty: true, markdown: '' });
		expect(labels(items)).toContain('Couper (off)');
		expect(labels(items)).toContain('Transformer en (off)');
		expect(labels(items)).toContain('Supprimer (off)');
		expect(labels(items)).toContain('Copier le lien du bloc');
		expect(labels(items)).toContain('Demander à Nova (off)');
	});
});
