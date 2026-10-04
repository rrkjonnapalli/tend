-- Schema v2: task labels. A fixed list kept in Settings → Tasks; a task can carry several.
-- Unlike tags (shared with notes, typed freely), labels are only for tasks and only come from
-- this list. Renaming a label renames it on every task (tasks link by id).

CREATE TABLE task_labels (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
  color      TEXT NOT NULL CHECK (color GLOB '#[0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F]'),
  position   INTEGER NOT NULL,
  created_at INTEGER NOT NULL
) STRICT;

CREATE TABLE task_label_links (
  task_id  TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  label_id TEXT NOT NULL REFERENCES task_labels(id) ON DELETE CASCADE,
  PRIMARY KEY (task_id, label_id)
) STRICT;

CREATE INDEX task_label_links_label ON task_label_links (label_id);
