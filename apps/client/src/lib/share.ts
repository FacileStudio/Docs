import type { EncreNode } from '@facile/encre';
import { backend } from './backend';

/** A share link just created: its id, to revoke it, and its full URL, shown once. */
export interface ShareLink {
	id: string;
	url: string;
}

/** The absolute URL of a share link, whether the API answers a path or a full URL. */
export function absoluteUrl(url: string, origin: string): string {
	return new URL(url, origin).toString();
}

/** Creates a public link to document `id`, with its subtree when `children` is set. */
export async function createShare(id: string, children: boolean, origin: string): Promise<ShareLink> {
	const created = await backend.share(id, children);
	return { id: created.id, url: absoluteUrl(created.url, origin) };
}

/** The document with every attachment image carrying `?share=token`, so a visitor without a session can load it. */
export function withShareToken(node: EncreNode, token: string): EncreNode {
	const src = node.attrs?.src;
	const attachment = node.type === 'image' && typeof src === 'string' && src.startsWith('/api/attachments/');
	const attrs = attachment
		? { ...node.attrs, src: `${src}${src.includes('?') ? '&' : '?'}share=${encodeURIComponent(token)}` }
		: node.attrs;
	const content = node.content?.map((child) => withShareToken(child, token));
	return { ...node, ...(attrs ? { attrs } : {}), ...(content ? { content } : {}) };
}
