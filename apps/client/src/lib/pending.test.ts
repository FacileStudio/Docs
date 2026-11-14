import { describe, expect, it } from 'vitest';
import { ApiError } from './backend';
import { failureMessage, needsBrowserLogin, targetDocument } from './pending';

describe('targetDocument', () => {
	it('reads the page of a docs_replace target and tolerates anything else', () => {
		expect(targetDocument({ target: '{"document_id":"d1","markdown":"x","expect_version":3}' })).toBe('d1');
		expect(targetDocument({ target: '{"document_id":4}' })).toBeNull();
		expect(targetDocument({ target: 'not json' })).toBeNull();
	});
});

describe('needsBrowserLogin', () => {
	it('is the 403 permission_denied only', () => {
		expect(needsBrowserLogin(new ApiError(403, 'permission_denied', 'only the person’s own session'))).toBe(true);
		expect(needsBrowserLogin(new ApiError(404, 'not_found', 'gone'))).toBe(false);
		expect(needsBrowserLogin(new ApiError(401, 'unauthenticated', 'missing auth'))).toBe(false);
		expect(needsBrowserLogin(new Error('boom'))).toBe(false);
		expect(failureMessage(new ApiError(404, 'not_found', 'gone'), 'x')).toBe('gone');
		expect(failureMessage('weird', 'x')).toBe('x');
	});
});
