import type { Editor } from '@tiptap/core';
import { backend } from './backend';

/** The image types the API accepts for attachments (plan 8.2). SVG is refused there, so it is skipped here. */
export const IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];

/** The largest attachment the API accepts. */
export const MAX_BYTES = 20 * 1024 * 1024;

/** The image files a paste or a drop carries. */
export function imageFiles(data: DataTransfer | null): File[] {
	return [...(data?.files ?? [])].filter((file) => IMAGE_TYPES.includes(file.type));
}

/** Uploads each image to the document's attachments and inserts it at the cursor, in order. */
export async function insertImages(editor: Editor, documentId: string, files: File[]): Promise<number> {
	let inserted = 0;
	for (const file of files) {
		if (file.size > MAX_BYTES) continue;
		const { url } = await backend.upload(documentId, file);
		editor
			.chain()
			.focus()
			.setImage({ src: url, alt: file.name.replace(/\.[^.]+$/, '') })
			.run();
		inserted += 1;
	}
	return inserted;
}
