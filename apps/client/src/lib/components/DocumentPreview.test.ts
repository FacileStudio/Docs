import { flushSync, mount, unmount } from 'svelte';
import { afterEach, describe, expect, it } from 'vitest';
import DocumentPreview from './DocumentPreview.svelte';

let app: ReturnType<typeof mount> | null = null;

afterEach(() => {
	if (app) unmount(app);
	app = null;
	document.body.innerHTML = '';
});

function renderPreview(props: { icon?: string | null }) {
	const target = document.createElement('div');
	document.body.append(target);
	app = mount(DocumentPreview, { target, props });
	flushSync();
	return target;
}

describe('DocumentPreview', () => {
	it('renders document mockup with lines', () => {
		const target = renderPreview({ icon: 'book' });
		const preview = target.querySelector('[aria-hidden="true"]');
		expect(preview).not.toBeNull();
		const lines = target.querySelectorAll('.bg-fc-component');
		expect(lines.length).toBeGreaterThanOrEqual(2);
	});
});
