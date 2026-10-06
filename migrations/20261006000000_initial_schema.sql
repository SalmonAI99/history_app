-- Baseline schema currently created by src/db.rs::init().
-- This migration includes the current column definitions so it can create a
-- fresh database. Existing databases must already have received the old
-- init() column upgrades before this baseline is applied: SQLite's
-- CREATE TABLE IF NOT EXISTS does not add columns to an existing table.

CREATE TABLE IF NOT EXISTS books (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    title   TEXT NOT NULL,
    author  TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS chapters (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id  INTEGER NOT NULL REFERENCES books(id),
    idx      INTEGER NOT NULL,
    title    TEXT NOT NULL,
    content  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role          TEXT NOT NULL DEFAULT 'student'
);

CREATE TABLE IF NOT EXISTS sessions (
    token      TEXT PRIMARY KEY,
    user_id    INTEGER NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS files (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id       INTEGER NOT NULL REFERENCES users(id),
    original_name TEXT NOT NULL,
    stored_name   TEXT NOT NULL,
    mime_type     TEXT NOT NULL DEFAULT '',
    size_bytes    INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS posts (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    kind         TEXT NOT NULL DEFAULT 'blog',
    lang         TEXT NOT NULL DEFAULT 'zh',
    slug         TEXT NOT NULL UNIQUE,
    title        TEXT NOT NULL,
    content      TEXT NOT NULL,
    published_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS subscriptions (
    user_id    INTEGER PRIMARY KEY REFERENCES users(id),
    paid_until TEXT,
    provider   TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS classes (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT NOT NULL,
    notes      TEXT NOT NULL DEFAULT '',
    syllabus   TEXT NOT NULL DEFAULT '',
    content    TEXT NOT NULL DEFAULT '',
    videos     TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS class_comments (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    class_id   INTEGER NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    user_id    INTEGER NOT NULL REFERENCES users(id),
    content    TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS course_access (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL REFERENCES users(id),
    deck_id    TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, deck_id)
);

CREATE TABLE IF NOT EXISTS learner_profiles (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_user_id  INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    linked_user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    name           TEXT NOT NULL,
    age_range      TEXT NOT NULL DEFAULT '',
    gender         TEXT NOT NULL DEFAULT '',
    created_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS learner_profiles_linked_user_unique
    ON learner_profiles(linked_user_id)
    WHERE linked_user_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS account_link_tokens (
    token           TEXT PRIMARY KEY,
    learner_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at      TEXT NOT NULL,
    used_at         TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS course_purchases (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    course_id  INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, course_id)
);

CREATE TABLE IF NOT EXISTS learner_courses (
    learner_profile_id INTEGER NOT NULL REFERENCES learner_profiles(id) ON DELETE CASCADE,
    course_id           INTEGER NOT NULL,
    assigned_at         TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY(learner_profile_id, course_id)
);

CREATE TABLE IF NOT EXISTS course_progress (
    learner_profile_id INTEGER NOT NULL REFERENCES learner_profiles(id) ON DELETE CASCADE,
    course_id           INTEGER NOT NULL,
    deck_id             TEXT NOT NULL,
    last_slide          INTEGER NOT NULL DEFAULT 1,
    updated_at          TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY(learner_profile_id, course_id, deck_id)
);

-- Preserve the startup backfill for student profiles.
INSERT INTO learner_profiles (owner_user_id, name)
SELECT u.id, u.username
FROM users AS u
WHERE u.role = 'student'
  AND NOT EXISTS (
      SELECT 1
      FROM learner_profiles AS lp
      WHERE lp.owner_user_id = u.id
  );

-- Preserve the old deck-grant conversion using the current course/deck map.
-- INSERT OR IGNORE makes this safe for databases that already ran the old
-- startup conversion.
INSERT OR IGNORE INTO course_purchases (user_id, course_id)
SELECT ca.user_id, deck_courses.course_id
FROM course_access AS ca
JOIN (
    SELECT 1 AS course_id, 'israel_palestine_1' AS deck_id
    UNION ALL SELECT 1, 'israel_palestine_2'
    UNION ALL SELECT 2, 'toilet14'
) AS deck_courses ON deck_courses.deck_id = ca.deck_id;

INSERT OR IGNORE INTO learner_courses (learner_profile_id, course_id)
SELECT lp.id, cp.course_id
FROM learner_profiles AS lp
JOIN course_purchases AS cp ON cp.user_id = lp.owner_user_id
WHERE cp.course_id IN (1, 2);
