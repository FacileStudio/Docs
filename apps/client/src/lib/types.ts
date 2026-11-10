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

/** A document as the API answers it. `content` or `markdown` depends on `?format=`. */
export interface DocumentRecord extends TreeNode {
	space_id: string | null;
	schema_version: number;
	content?: EncreNode;
	markdown?: string;
	created_at: string;
	trashed_at: string | null;
}

/** One saved revision of a document. */
export interface Revision {
	id: string;
	title: string;
	created_at: string;
	created_by?: number | null;
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

/** A child page listed by a share link that exposes its subtree. */
export interface PublicChild {
	id: string;
	title: string;
	parent_id?: string | null;
	icon?: string;
}

/** A shared page, as `GET /api/public/{token}` answers it. */
export interface PublicPage {
	title: string;
	content: EncreNode;
	children?: PublicChild[];
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
