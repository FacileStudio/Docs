ALTER TABLE spaces ADD COLUMN deleted_at timestamptz;
CREATE INDEX documents_trash_idx ON documents (trashed_at) WHERE trashed_at IS NOT NULL;
