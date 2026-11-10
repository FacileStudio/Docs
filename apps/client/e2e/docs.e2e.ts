import { afterAll, beforeAll, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { chromium, type Browser, type Page } from 'playwright-core';
import { editor, imageLoaded, insertCallout, live, pasteImage, seesWithin, typeAtEnd } from './editor';
import { startStack, type Person, type Stack } from './stack';

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

async function pageAs(who: Person | null, path: string): Promise<Page> {
	const context = await browser.newContext({ acceptDownloads: true });
	if (who) await context.addCookies([{ name: 'session', value: stack.cookies[who], url: stack.base }]);
	const page = await context.newPage();
	await page.goto(`${stack.base}${path}`);
	return page;
}

async function api<T>(path: string, init: { method?: string; body?: unknown } = {}): Promise<T> {
	const answer = await fetch(`${stack.base}/api${path}`, {
		method: init.method ?? 'GET',
		headers: { cookie: `session=${stack.cookies.alice}`, 'content-type': 'application/json', 'x-facile-csrf': '1' },
		body: init.body === undefined ? undefined : JSON.stringify(init.body)
	});
	if (!answer.ok) throw new Error(`${path}: ${answer.status} ${await answer.text()}`);
	return (await answer.json()) as T;
}

async function persisted(id: string, needles: string[]): Promise<string> {
	for (let attempt = 0; attempt < 50; attempt += 1) {
		const { markdown } = await api<{ markdown: string }>(`/documents/${id}?format=markdown`);
		if (needles.every((needle) => markdown.includes(needle))) return markdown;
		await Bun.sleep(200);
	}
	throw new Error(`the stored page never held ${needles.join(', ')}`);
}

test('two members write together, and the page survives a reload, a share, an outsider and a PDF', async () => {
	const { id } = await api<{ id: string }>('/documents', {
		method: 'POST',
		body: { title: 'Plan partagé', space_id: stack.space }
	});
	const [alice, bob] = await Promise.all([pageAs('alice', `/d/${id}`), pageAs('bob', `/d/${id}`)]);
	await Promise.all([live(alice), live(bob)]);

	await typeAtEnd(alice, 'Bonjour de la part d’Alice');
	expect(await seesWithin(bob, 'Bonjour de la part d’Alice', 1_000)).toBeLessThan(1_000);
	await typeAtEnd(bob, ', et Bob répond');
	expect(await seesWithin(alice, 'et Bob répond', 1_000)).toBeLessThan(1_000);

	await insertCallout(alice, 'Attention au budget');
	await bob.locator('[data-encre-node="callout"]').getByText('Attention au budget').waitFor({ timeout: 5_000 });

	await alice.keyboard.press('Enter');
	await pasteImage(alice);
	const src = await imageLoaded(alice);
	expect(src).toMatch(/^\/api\/attachments\/[0-9a-f-]{36}$/);
	await imageLoaded(bob);

	await bob.context().close();
	await alice.reload();
	await live(alice);
	await editor(alice).getByText('et Bob répond').waitFor();
	await alice.locator('[data-encre-node="callout"]').getByText('Attention au budget').waitFor();
	await imageLoaded(alice);
	await persisted(id, ['Bonjour', 'et Bob répond', 'Attention au budget', src]);

	const download = alice.waitForEvent('download');
	await alice.getByRole('button', { name: 'Exporter' }).click();
	await alice.getByRole('menuitem', { name: 'PDF' }).click();
	const pdf = readFileSync((await (await download).path()) ?? '');
	expect(pdf.subarray(0, 5).toString()).toBe('%PDF-');
	expect(pdf.length).toBeGreaterThan(1_000);

	const share = await api<{ url: string }>(`/documents/${id}/share`, { method: 'POST', body: { children: false } });
	const visitor = await pageAs(null, share.url);
	expect(await visitor.context().cookies()).toEqual([]);
	await visitor.getByRole('heading', { name: 'Plan partagé' }).waitFor();
	await visitor.getByText('et Bob répond').waitFor();
	await visitor.getByText('Attention au budget').waitFor();
	expect(await imageLoaded(visitor, visitor.locator('main, body'))).toContain('?share=');
	expect(await visitor.locator('meta[name="robots"]').getAttribute('content')).toContain('noindex');

	const carol = await pageAs('carol', `/d/${id}`);
	await carol.getByText('Document introuvable').waitFor({ timeout: 10_000 });
	expect(await carol.locator('.ProseMirror').count()).toBe(0);
	const denied = await fetch(`${stack.base}/api/documents/${id}`, { headers: { cookie: `session=${stack.cookies.carol}` } });
	expect(denied.status).toBe(404);
}, 180_000);
