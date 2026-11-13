import type { Browser, Page } from 'playwright-core';
import type { Person, Stack } from './stack';

/** A French browser page at `path`, signed in as `who` (or nobody). */
export async function pageAs(browser: Browser, stack: Stack, who: Person | null, path: string): Promise<Page> {
	const context = await browser.newContext({ acceptDownloads: true, locale: 'fr-FR' });
	if (who) await context.addCookies([{ name: 'session', value: stack.cookies[who], url: stack.base }]);
	const page = await context.newPage();
	await page.goto(`${stack.base}${path}`);
	return page;
}

/** Calls the API as Alice, with the CSRF header a browser write carries. */
export async function apiAs<T>(stack: Stack, path: string, init: { method?: string; body?: unknown } = {}): Promise<T> {
	const answer = await fetch(`${stack.base}/api${path}`, {
		method: init.method ?? 'GET',
		headers: { cookie: `session=${stack.cookies.alice}`, 'content-type': 'application/json', 'x-facile-csrf': '1' },
		body: init.body === undefined ? undefined : JSON.stringify(init.body)
	});
	if (!answer.ok) throw new Error(`${path}: ${answer.status} ${await answer.text()}`);
	return (await answer.json()) as T;
}
