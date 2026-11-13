import type { EncreNode } from '@facile/encre';

/** A role in a Registre space, as `GET /api/spaces` reports it. */
export type Role = 'member' | 'admin' | 'owner';

/** A space the caller belongs to. */
export interface Space {
	id: string;
	name: string;
	role: Role;
}

/** One row of `GET /api/documents`: the tree, without content. */
export interface TreeNode {
	id: string;
	parent_id: string | null;
	position: number;
	title: string;
	icon: string;
	updated_at: string;
	trashed_at?: string | null;
}

/** What the caller may do with a document: read, edit content and title, or also move, trash and share. */
export type Access = 'viewer' | 'editor' | 'manager';

/** A document as the API answers it. `content` or `markdown` depends on `?format=`. */
export interface DocumentRecord extends TreeNode {
	space_id: string | null;
	schema_version: number;
	version: number;
	access: Access;
	content?: EncreNode;
	markdown?: string;
	created_at: string;
	trashed_at: string | null;
}

/** One row of `GET /api/documents/{id}/revisions`: a saved revision, without its body. */
export interface Revision {
	id: string;
	title: string;
	schema_version: number;
	created_by: number | null;
	author: string | null;
	created_at: string;
}

/** One search result. `snippet` comes from `ts_headline` and marks hits with `<b>`. */
export interface SearchHit {
	id: string;
	title: string;
	snippet: string;
	updated_at: string;
}

/** What the API answers when it creates an attachment or a share link. */
export interface Created {
	id: string;
	url: string;
}

/** A shared page, as `GET /api/public/{token}` answers it. `children` is the live subtree, empty on a child page. */
export interface PublicPage {
	id: string;
	title: string;
	icon: string;
	content: EncreNode;
	children: TreeNode[];
}

/** A live share link of a document, as `GET /api/documents/{id}/share` lists it. The token is never listed. */
export interface ShareInfo {
	id: string;
	children: boolean;
	created_at: string;
}

/** An agent action waiting for the owner's confirmation, as `GET /api/mcp/actions` lists it. */
export interface PendingAction {
	id: string;
	kind: string;
	target: string;
	summary: string;
	client_id: string;
	created_at: string;
	expires_at: string;
	state: string;
}

/** The signed-in person, as `GET /api/auth/session` answers it. */
export interface SessionUser {
	email: string;
	name?: string;
	avatar_url?: string;
}

/** Where a document is placed: its parent, and the sibling it goes before (`null` is the end). */
export interface Placement {
	parent_id: string | null;
	before: string | null;
}

/** What `POST /documents/{id}/attachments` answers: `src` is what an image or file block holds. */
export interface Uploaded {
	id: string;
	url: string;
	src: string;
	name: string;
	size: number;
	mime: string;
}

/** What `GET /embeds/resolve` answers for a link card. */
export interface EmbedAnswer {
	title: string;
	description: string | null;
	image: string | null;
	site: string | null;
	favicon: string | null;
}

/** The person's Bulle assistant, from `GET /assistant`. */
export interface AssistantProfile {
	name: string;
	accent: string | null;
}
