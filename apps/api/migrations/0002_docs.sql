CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE spaces (
	id         text PRIMARY KEY,
	name       text NOT NULL,
	updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE space_members (
	space_id text NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
	subject  text NOT NULL,
	email    text NOT NULL,
	name     text NOT NULL DEFAULT '',
	picture  text NOT NULL DEFAULT '',
	role     text NOT NULL CHECK (role IN ('member', 'admin', 'owner')),
	PRIMARY KEY (space_id, subject)
);
CREATE INDEX space_members_subject_idx ON space_members (subject);

CREATE TABLE directory_state (
	id      smallint PRIMARY KEY CHECK (id = 1),
	version bigint NOT NULL
);

CREATE TABLE deletion_state (
	id         smallint PRIMARY KEY CHECK (id = 1),
	state      bytea NOT NULL,
	updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE documents (
	id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	space_id   text REFERENCES spaces(id) ON DELETE CASCADE,
	owner_id   bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	parent_id  uuid REFERENCES documents(id) ON DELETE CASCADE,
	position   double precision NOT NULL DEFAULT 0,
	title      text NOT NULL DEFAULT '',
	icon       text NOT NULL DEFAULT '',
	schema_version integer NOT NULL DEFAULT 1,
	version    bigint NOT NULL DEFAULT 1,
	content    jsonb NOT NULL,
	text       text NOT NULL DEFAULT '',
	ydoc       bytea NOT NULL DEFAULT ''::bytea,
	created_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	updated_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at timestamptz NOT NULL DEFAULT now(),
	updated_at timestamptz NOT NULL DEFAULT now(),
	trashed_at timestamptz,
	search     tsvector GENERATED ALWAYS AS (
		setweight(to_tsvector('simple', title), 'A') || setweight(to_tsvector('simple', text), 'B')
	) STORED,
	CHECK (space_id IS NOT NULL OR owner_id IS NOT NULL)
);
CREATE INDEX documents_tree_idx ON documents (space_id, parent_id, position) WHERE trashed_at IS NULL;
CREATE INDEX documents_owner_idx ON documents (owner_id, parent_id, position) WHERE space_id IS NULL AND trashed_at IS NULL;
CREATE INDEX documents_search_idx ON documents USING gin (search);

CREATE TABLE document_revisions (
	id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	document_id uuid NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
	title          text NOT NULL,
	schema_version integer NOT NULL,
	content        jsonb NOT NULL,
	created_by     bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at     timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX document_revisions_idx ON document_revisions (document_id, created_at DESC);

CREATE TABLE attachments (
	id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	document_id uuid NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
	name        text NOT NULL,
	mime        text NOT NULL,
	size        bigint NOT NULL,
	sha256      text NOT NULL,
	created_by  bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at  timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE share_links (
	id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	document_id uuid NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
	token_hash  text NOT NULL UNIQUE,
	children    boolean NOT NULL DEFAULT false,
	created_by  bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at  timestamptz NOT NULL DEFAULT now(),
	revoked_at  timestamptz
);
