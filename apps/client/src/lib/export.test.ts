import { describe, expect, it, vi } from 'vitest';
import { exportUrl, runExport } from './export';

describe('export flow', () => {
	it('downloads every format from the API', () => {
		const open = vi.fn();
		runExport('d 1', 'pdf', open);
		runExport('d 1', 'markdown', open);
		runExport('d 1', 'commonmark', open);
		expect(open.mock.calls).toEqual([
			['/api/documents/d%201/export.pdf'],
			['/api/documents/d%201/export.md'],
			['/api/documents/d%201/export.commonmark']
		]);
	});

	it('points CommonMark at its own route', () => {
		expect(exportUrl('d', 'commonmark')).toBe('/api/documents/d/export.commonmark');
	});
});
