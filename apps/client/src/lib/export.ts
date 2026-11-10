import { backend } from './backend';

/** The three export formats of the document menu, all rendered by the API. */
export type ExportKind = 'pdf' | 'markdown' | 'commonmark';

const ROUTE = { pdf: 'pdf', markdown: 'md', commonmark: 'commonmark' } as const;

/** The API download of document `id` in format `kind`. */
export function exportUrl(id: string, kind: ExportKind): string {
	return backend.exportUrl(id, ROUTE[kind]);
}

/** Asks the browser to download `url`, keeping the server's file name. */
export function download(url: string): void {
	const link = document.createElement('a');
	link.href = url;
	link.download = '';
	link.click();
}

/** Runs one export: the browser downloads it from the API, which names the file. */
export function runExport(id: string, kind: ExportKind, open: (url: string) => void = download): void {
	open(exportUrl(id, kind));
}
