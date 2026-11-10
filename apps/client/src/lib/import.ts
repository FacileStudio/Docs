import { loadEncre, parseMarkdown, type EncreNode } from '@facile/encre';
import { backend } from './backend';
import type { DocumentRecord } from './types';

/** The title of an imported file: its name without the extension. */
export function titleOf(fileName: string): string {
	return fileName.replace(/\.(md|markdown|txt)$/i, '').trim() || 'Sans titre';
}

/** Reads a markdown file through encre-wasm and creates a document from it in the active scope. */
export async function importMarkdown(file: File, spaceId: string | null, parentId?: string): Promise<DocumentRecord> {
	await loadEncre();
	const content: EncreNode = parseMarkdown(await file.text());
	return backend.create({
		title: titleOf(file.name),
		content,
		...(spaceId ? { space_id: spaceId } : {}),
		...(parentId ? { parent_id: parentId } : {})
	});
}
