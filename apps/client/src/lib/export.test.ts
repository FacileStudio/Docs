import { afterEach, describe, expect, it, vi } from 'vitest';
import { fileName, runExport, serverExport } from './export';

afterEach(() => vi.unstubAllGlobals());

describe('export flow', () => {
	it('downloads PDF and markdown from the API', async () => {
		const open = vi.fn();
		const commonMark = vi.fn(async () => 'never');
		await runExport('d 1', 'Guide', 'pdf', commonMark, open);
		await runExport('d 1', 'Guide', 'markdown', commonMark, open);
		expect(open.mock.calls).toEqual([['/api/documents/d%201/export.pdf'], ['/api/documents/d%201/export.md']]);
		expect(commonMark).not.toHaveBeenCalled();
	});

	it('renders CommonMark in the browser and saves it under the title', async () => {
		const blobs: Blob[] = [];
		vi.stubGlobal(
			'URL',
			Object.assign(URL, { createObjectURL: (blob: Blob) => (blobs.push(blob), 'blob:x'), revokeObjectURL: vi.fn() })
		);
		const click = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(function (this: HTMLAnchorElement) {
			expect(this.download).toBe('Plan Q3.md');
		});
		const open = vi.fn();
		await runExport('d1', 'Plan: Q3', 'commonmark', async () => '# Plan\n', open);
		expect(open).not.toHaveBeenCalled();
		expect(click).toHaveBeenCalledOnce();
		expect(await blobs[0].text()).toBe('# Plan\n');
	});

	it('only the portable form is rendered client-side', () => {
		expect(serverExport('d', 'commonmark')).toBeNull();
	});

	it('makes safe file names', () => {
		expect(fileName('a/b\\c: "d"', 'pdf')).toBe('a b c d.pdf');
		expect(fileName('   ', 'md')).toBe('document.md');
	});
});
