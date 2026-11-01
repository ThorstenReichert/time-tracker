-- R24: optional abbreviations for clients, projects and tasks (changes/2026-11-01-abbreviations).
-- Names and abbreviations share one namespace per parent (R1). The core checks all of it;
-- these indexes are the backstop for abbreviation against abbreviation. A name clashing with
-- a sibling's abbreviation has no database backstop.

ALTER TABLE client ADD COLUMN abbreviation TEXT;
ALTER TABLE project ADD COLUMN abbreviation TEXT;
ALTER TABLE task ADD COLUMN abbreviation TEXT;

CREATE UNIQUE INDEX client_abbreviation ON client (abbreviation COLLATE NOCASE)
    WHERE abbreviation IS NOT NULL;
CREATE UNIQUE INDEX project_abbreviation ON project (client_id, abbreviation COLLATE NOCASE)
    WHERE abbreviation IS NOT NULL;
CREATE UNIQUE INDEX task_abbreviation ON task (project_id, abbreviation COLLATE NOCASE)
    WHERE abbreviation IS NOT NULL;
