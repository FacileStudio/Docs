import type { UploadedImage } from '@facile/encre';
import { backend } from './backend';

/** The largest attachment the API accepts. */
export const MAX_BYTES = 20 * 1024 * 1024;

/** The alt text of an uploaded image: its file name without the extension. */
export function altOf(name: string): string {
	return name.replace(/\.[^.]+$/, '');
}

/** The encre `onimage` hook of document `documentId`: uploads the file to its attachments. */
export function uploader(documentId: string): (file: File) => Promise<UploadedImage> {
	return async (file) => {
		const { url } = await backend.upload(documentId, file);
		return { src: url, alt: altOf(file.name) };
	};
}
