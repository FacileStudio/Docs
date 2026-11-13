import type { Page } from 'playwright-core';

/** The encre editor of a page. */
export const editor = (page: Page) => page.locator('.ProseMirror');

/** Waits until the page shows its editor and the room is live. */
export async function live(page: Page): Promise<void> {
	await editor(page).waitFor();
	await page.locator('[data-connection="live"]').waitFor({ timeout: 15_000 });
}

/** Waits for `text` in the editor and answers how long it took, in milliseconds. */
export async function seesWithin(page: Page, text: string, within: number): Promise<number> {
	const start = performance.now();
	await editor(page).getByText(text).waitFor({ timeout: within });
	return performance.now() - start;
}

/** Types `text` at the end of the document. */
export async function typeAtEnd(page: Page, text: string): Promise<void> {
	await editor(page).click();
	await page.keyboard.press('Control+End');
	await page.keyboard.type(text);
}

/** Starts a new block, turns it into a callout through the slash menu and types `text` in it. */
export async function insertCallout(page: Page, text: string): Promise<void> {
	await page.keyboard.press('Enter');
	await page.keyboard.type('/encadr');
	await page.getByRole('option', { name: 'Encadré' }).click();
	await page.keyboard.type(text);
}

/** Pastes `markdown` as plain text, as the clipboard would; encre reads it as Facile Markdown. */
export async function pasteMarkdown(page: Page, markdown: string): Promise<void> {
	await editor(page).evaluate((target, text) => {
		const data = new DataTransfer();
		data.setData('text/plain', text);
		target.dispatchEvent(new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true }));
	}, markdown);
}

/** Pastes a small PNG drawn on a canvas into the editor, as the clipboard would. */
export async function pasteImage(page: Page): Promise<void> {
	await editor(page).evaluate(async (target) => {
		const canvas = document.createElement('canvas');
		canvas.width = 24;
		canvas.height = 16;
		const context = canvas.getContext('2d');
		if (context) context.fillRect(0, 0, 24, 16);
		const blob = await new Promise<Blob | null>((done) => canvas.toBlob(done, 'image/png'));
		const data = new DataTransfer();
		data.items.add(new File([blob ?? new Blob()], 'schema.png', { type: 'image/png' }));
		target.dispatchEvent(new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true }));
	});
}

/** Waits for an attachment image in `scope` that the browser decoded, and answers its `src`. */
export async function imageLoaded(page: Page, scope = editor(page)): Promise<string> {
	const image = scope.locator('img[src*="/api/attachments/"]').first();
	await image.waitFor({ timeout: 10_000 });
	await page.waitForFunction((img) => (img as HTMLImageElement).naturalWidth > 0, await image.elementHandle());
	return (await image.getAttribute('src')) ?? '';
}
