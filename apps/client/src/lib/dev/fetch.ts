import { vi } from 'vitest';
import type { MockApi } from './mock-api.ts';

/** Routes every `fetch` of the test to `api`, recording the `RequestInit` each call carried. */
export function useMockFetch(api: MockApi) {
	const inits: RequestInit[] = [];
	const fetch = vi.fn(async (input: RequestInfo | URL, init: RequestInit = {}) => {
		inits.push(init);
		return api.handle(new Request(new URL(String(input), 'http://localhost'), init));
	});
	vi.stubGlobal('fetch', fetch);
	return { fetch, inits };
}
