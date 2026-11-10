import { describe, expect, it } from 'vitest';
import { targetDocument } from './pending';

describe('targetDocument', () => {
	it('reads the page of a docs_replace target and tolerates anything else', () => {
		expect(targetDocument({ target: '{"document_id":"d1","markdown":"x","expect_version":3}' })).toBe('d1');
		expect(targetDocument({ target: '{"document_id":4}' })).toBeNull();
		expect(targetDocument({ target: 'not json' })).toBeNull();
	});
});
