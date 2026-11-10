import { describe, expect, it } from 'vitest';
import { rememberReturn, takeReturn } from './return';

function memory() {
	const items = new Map<string, string>();
	return {
		getItem: (key: string) => items.get(key) ?? null,
		setItem: (key: string, value: string) => void items.set(key, value),
		removeItem: (key: string) => void items.delete(key)
	};
}

describe('sign-in return path', () => {
	it('brings a signed-out visitor back to the pending action once', () => {
		const store = memory();
		rememberReturn(new URL('https://docs.test/pending?action=a1'), store);
		expect(takeReturn(store)).toBe('/pending?action=a1');
		expect(takeReturn(store)).toBeNull();
	});

	it('forgets the path on the home page and refuses anything that leaves the origin', () => {
		const store = memory();
		rememberReturn(new URL('https://docs.test/pending?action=a1'), store);
		rememberReturn(new URL('https://docs.test/'), store);
		expect(takeReturn(store)).toBeNull();
		store.setItem('docs.return', '//evil.test/x');
		expect(takeReturn(store)).toBeNull();
		store.setItem('docs.return', 'https://evil.test');
		expect(takeReturn(store)).toBeNull();
	});
});
