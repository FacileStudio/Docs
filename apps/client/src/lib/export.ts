import { backend } from './backend';

/** The three export formats of the document menu. */
export type ExportKind = 'pdf' | 'markdown' | 'commonmark';

/** A file name from a document title: no path separators or control characters, never empty. */
export function fileName(title: string, extension: string): string {
	const base = title
		.replace(/[\\/:*?"<>|\p{Cc}]+/gu, ' ')
		.replace(/\s+/g, ' ')
		.trim()
		.slice(0, 120);
	return `${base || 'document'}.${extension}`;
}

/** The server download for a format the API renders, or null for one the browser renders. */
export function serverExport(id: string, kind: ExportKind): string | null {
	if (kind === 'pdf') return backend.exportUrl(id, 'pdf');
	if (kind === 'markdown') return backend.exportUrl(id, 'md');
	return null;
}

/** Asks the browser to download `url` as `name` (empty keeps the server's file name). */
export function download(url: string, name = ''): void {
	const link = document.createElement('a');
	link.href = url;
	link.download = name;
	link.click();
}

/** Hands `text` to the browser as a file download. */
export function downloadText(name: string, text: string, type = 'text/markdown'): void {
	const url = URL.createObjectURL(new Blob([text], { type: `${type};charset=utf-8` }));
	download(url, name);
	setTimeout(() => URL.revokeObjectURL(url), 0);
}

/** Runs one export: a server download for PDF and markdown, the editor's CommonMark for the portable form. */
export async function runExport(
	id: string,
	title: string,
	kind: ExportKind,
	commonMark: () => Promise<string>,
	open: (url: string) => void = download
): Promise<void> {
	const url = serverExport(id, kind);
	if (url) return open(url);
	downloadText(fileName(title, 'md'), await commonMark());
}
