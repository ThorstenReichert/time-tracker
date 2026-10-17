-- Schema from spec.md §2 "Data model (SQLite)". The core enforces every rule itself; the
-- constraints here are a backstop. Instants are UTC Unix seconds.

CREATE TABLE client (
    id              INTEGER PRIMARY KEY,
    name            TEXT    NOT NULL,
    round_unit_min  INTEGER NOT NULL CHECK (round_unit_min IN (6, 12, 15, 30, 60)),
    round_direction TEXT    NOT NULL CHECK (round_direction IN ('up', 'nearest', 'down')),
    round_level     TEXT    NOT NULL CHECK (round_level IN ('project', 'client')),
    archived_at     INTEGER,
    created_at      INTEGER NOT NULL
) STRICT;

-- R1: names are unique within their parent, ignoring case; archived rows count too (OQ-7).
CREATE UNIQUE INDEX client_name ON client (name COLLATE NOCASE);

CREATE TABLE project (
    id          INTEGER PRIMARY KEY,
    client_id   INTEGER NOT NULL REFERENCES client (id),
    name        TEXT    NOT NULL,
    archived_at INTEGER,
    created_at  INTEGER NOT NULL
) STRICT;

CREATE UNIQUE INDEX project_name ON project (client_id, name COLLATE NOCASE);

CREATE TABLE task (
    id          INTEGER PRIMARY KEY,
    project_id  INTEGER NOT NULL REFERENCES project (id),
    name        TEXT    NOT NULL,
    archived_at INTEGER,
    created_at  INTEGER NOT NULL,
    UNIQUE (id, project_id)
) STRICT;

CREATE UNIQUE INDEX task_name ON task (project_id, name COLLATE NOCASE);

CREATE TABLE time_entry (
    id         INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project (id),
    task_id    INTEGER,
    start_utc  INTEGER NOT NULL,
    end_utc    INTEGER CHECK (end_utc IS NULL OR end_utc > start_utc),
    note       TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    -- The task belongs to the entry's project. A NULL task_id skips the check (MATCH SIMPLE).
    FOREIGN KEY (task_id, project_id) REFERENCES task (id, project_id)
) STRICT;

-- R4: at most one running entry. A partial index on a non-NULL expression, so the
-- uniqueness check actually applies.
CREATE UNIQUE INDEX one_running ON time_entry ((end_utc IS NULL)) WHERE end_utc IS NULL;

-- The named queries (OQ-9) look entries up by start.
CREATE INDEX time_entry_start ON time_entry (start_utc);

CREATE TABLE favorite (
    id         INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project (id),
    task_id    INTEGER,
    sort       INTEGER NOT NULL,
    FOREIGN KEY (task_id, project_id) REFERENCES task (id, project_id)
) STRICT;

-- hotkey, decimal_separator, recents_n, backup_dir, backup_keep
CREATE TABLE setting (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

-- ID counters, written in the same transaction as the rows, so IDs are never reused.
CREATE TABLE counter (
    name    TEXT    PRIMARY KEY,
    next_id INTEGER NOT NULL
) STRICT;

INSERT INTO counter (name, next_id) VALUES
    ('client', 1), ('project', 1), ('task', 1), ('entry', 1), ('favorite', 1);
