import { afterAll, beforeAll, expect, test } from 'bun:test';
import { chromium, type Browser, type Page } from 'playwright-core';
import { editor, live, pasteMarkdown, typeAtEnd } from './editor';
import { apiAs, pageAs } from './pages';
import { startStack, type Stack } from './stack';

let stack: Stack;
let browser: Browser;

beforeAll(async () => {
	stack = await startStack();
	browser = await chromium.launch({ headless: true });
}, 60_000);

afterAll(async () => {
	await browser?.close();
	await stack?.stop();
});

async function page(title: string): Promise<{ id: string; alice: Page }> {
	const { id } = await apiAs<{ id: string }>(stack, '/documents', {
		method: 'POST',
		body: { title, space_id: stack.space }
	});
	const alice = await pageAs(browser, stack, 'alice', `/d/${id}`);
	await live(alice);
	return { id, alice };
}

async function selectLine(alice: Page, text: string): Promise<void> {
	await typeAtEnd(alice, text);
	await alice.keyboard.press('Shift+Home');
}

const BULLE = 'https://bulle.facile.studio/embed/chat?';

test('the page has no toolbar and a bare icon; / turns a paragraph into a heading', async () => {
	const { alice } = await page('Minimal');
	expect(await alice.locator('[role="toolbar"]').count()).toBe(0);
	const icon = alice.getByTestId('page-icon');
	expect(await icon.evaluate((node) => getComputedStyle(node).backgroundColor)).toBe('rgba(0, 0, 0, 0)');
	await editor(alice).click();
	await alice.keyboard.type('/titre');
	await alice.getByRole('option', { name: 'Titre 1' }).click();
	await alice.keyboard.type('Un vrai titre');
	await editor(alice).locator('h1', { hasText: 'Un vrai titre' }).waitFor({ timeout: 5_000 });
	await alice.context().close();
}, 60_000);

test('a selection shows the bubble menu and asks the assistant in the sidebar', async () => {
	const { id, alice } = await page('Bulle');
	await selectLine(alice, 'Une phrase à expliquer');
	await alice.getByRole('button', { name: 'Transformer en' }).waitFor({ timeout: 5_000 });
	await alice.getByRole('button', { name: 'Demander à Bulle' }).click();
	const frame = alice.getByTestId('bulle-sidebar').locator('iframe');
	await frame.waitFor({ timeout: 5_000 });
	const src = (await frame.getAttribute('src')) ?? '';
	expect(src.startsWith(BULLE)).toBe(true);
	const query = new URL(src).searchParams;
	expect([query.get('app'), query.get('id'), query.get('url')]).toEqual(['docs', id, `${stack.base}/d/${id}`]);
	await alice.getByRole('button', { name: 'Fermer Bulle' }).click();
	await alice.getByTestId('bulle-sidebar').waitFor({ state: 'hidden' });
	await alice.getByTestId('bulle-toggle').click();
	await alice.getByTestId('bulle-sidebar').waitFor();
	await alice.context().close();
}, 60_000);

test('a right click opens the Docs menu, and a pasted link becomes an embed', async () => {
	const { alice } = await page('Menu');
	await typeAtEnd(alice, 'Bloc de départ');
	await editor(alice).getByText('Bloc de départ').click({ button: 'right' });
	const menu = alice.getByRole('menu');
	await menu.getByRole('menuitem', { name: 'Transformer en' }).waitFor({ timeout: 5_000 });
	expect(await menu.getByRole('menuitem', { name: 'Demander à Bulle' }).count()).toBe(1);
	await alice.keyboard.press('Escape');
	await typeAtEnd(alice, '');
	await alice.keyboard.press('Enter');
	await pasteMarkdown(alice, 'https://example.com/article');
	await editor(alice).locator('[data-encre-node="embed"]').waitFor({ timeout: 5_000 });
	await alice.context().close();
}, 60_000);

test('the right-click menu opens without a ring and arrows scroll its last entry into view', async () => {
	const { alice } = await page('Clavier');
	await typeAtEnd(alice, 'Bloc à transformer');
	await editor(alice).getByText('Bloc à transformer').click({ button: 'right' });
	const menu = alice.getByRole('menu');
	await menu.getByRole('menuitem', { name: 'Transformer en' }).click();
	const entries = menu.getByRole('menuitem');
	const count = await entries.count();
	expect(await alice.evaluate(() => document.activeElement?.getAttribute('role'))).toBe('menu');
	for (let i = 0; i < count; i++) await alice.keyboard.press('ArrowDown');
	const last = entries.last();
	expect(await last.evaluate((node) => node === document.activeElement)).toBe(true);
	const visible = await last.evaluate((node) => {
		const box = node.getBoundingClientRect();
		const frame = (node.closest('[role="menu"]') as HTMLElement).getBoundingClientRect();
		return box.top >= frame.top && box.bottom <= frame.bottom && box.bottom <= window.innerHeight;
	});
	expect(visible).toBe(true);
	expect(await menu.evaluate((node) => node.scrollTop > 0)).toBe(true);
	await alice.context().close();
}, 60_000);

test('presence shows nobody alone and the other person once they join', async () => {
	const { id, alice } = await page('Ensemble');
	const region = alice.getByRole('group', { name: 'Personnes connectées' });
	expect(await region.count()).toBe(0);
	const bob = await pageAs(browser, stack, 'bob', `/d/${id}`);
	await live(bob);
	await region.waitFor({ timeout: 10_000 });
	await bob.context().close();
	await region.waitFor({ state: 'detached', timeout: 60_000 });
	await alice.context().close();
}, 120_000);
