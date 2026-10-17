-- Habit & Task Planner: initial schema.
-- All timestamps are TEXT in ISO-8601 UTC ("2026-10-16T08:30:00Z"); local_date is "YYYY-MM-DD".
-- Columns are declared TEXT on purpose: sqlx maps TEXT <-> Rust String directly
-- (a column declared TIMESTAMP/DATE would NOT decode into String).

CREATE TABLE categories (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL UNIQUE,
    color      TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE tasks (
    id                         INTEGER PRIMARY KEY AUTOINCREMENT,
    title                      TEXT NOT NULL,
    description                TEXT,
    status                     TEXT NOT NULL DEFAULT 'todo'
                               CHECK (status IN ('todo','in_progress','done','archived')),
    priority                   TEXT NOT NULL DEFAULT 'medium'
                               CHECK (priority IN ('low','medium','high','urgent')),
    category_id                INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    due_at                     TEXT,
    scheduled_start_at         TEXT,
    estimated_duration_minutes INTEGER CHECK (estimated_duration_minutes IS NULL OR estimated_duration_minutes > 0),
    recurrence_rule            TEXT,
    created_at                 TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at                 TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    completed_at               TEXT,
    archived_at                TEXT
);

CREATE TABLE subtasks (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id      INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    title        TEXT NOT NULL,
    is_completed BOOLEAN NOT NULL DEFAULT 0,
    position     INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    completed_at TEXT
);

CREATE TABLE habits (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    title            TEXT NOT NULL,
    description      TEXT,
    category_id      INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    frequency_type   TEXT NOT NULL DEFAULT 'daily'
                     CHECK (frequency_type IN ('daily','weekly','custom')),
    frequency_config TEXT,
    target_count     INTEGER CHECK (target_count IS NULL OR target_count > 0),
    reminder_time    TEXT,
    is_active        BOOLEAN NOT NULL DEFAULT 1,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    archived_at      TEXT
);

CREATE TABLE habit_completions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    habit_id        INTEGER NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
    local_date      TEXT NOT NULL,
    completed_count INTEGER NOT NULL DEFAULT 1,
    note            TEXT,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    UNIQUE (habit_id, local_date)
);

CREATE TABLE goals (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    title        TEXT NOT NULL,
    description  TEXT,
    target_date  TEXT,
    status       TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','completed','archived')),
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    completed_at TEXT
);

CREATE TABLE goal_milestones (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    goal_id      INTEGER NOT NULL REFERENCES goals(id) ON DELETE CASCADE,
    title        TEXT NOT NULL,
    target_date  TEXT,
    status       TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','completed','archived')),
    completed_at TEXT
);

CREATE TABLE scheduled_blocks (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id    INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    title      TEXT NOT NULL,
    start_at   TEXT NOT NULL,
    end_at     TEXT NOT NULL,
    block_type TEXT NOT NULL DEFAULT 'task',
    status     TEXT NOT NULL DEFAULT 'scheduled',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- Ledger of every XP change. Rows are never deleted: undoing an action adds a negative row.
CREATE TABLE xp_transactions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    amount      INTEGER NOT NULL,
    source_type TEXT NOT NULL,
    source_id   INTEGER NOT NULL,
    reason      TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- Single-row table (id is forced to 1).
CREATE TABLE player_profile (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    current_level INTEGER NOT NULL DEFAULT 1,
    total_xp      INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
INSERT INTO player_profile (id) VALUES (1);

CREATE TABLE achievements (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    description TEXT,
    icon        TEXT,
    xp_reward   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE user_achievements (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    achievement_id INTEGER NOT NULL UNIQUE REFERENCES achievements(id) ON DELETE CASCADE,
    unlocked_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE quests (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    title        TEXT NOT NULL,
    description  TEXT,
    quest_type   TEXT NOT NULL CHECK (quest_type IN ('daily','weekly')),
    target_value INTEGER NOT NULL,
    xp_reward    INTEGER NOT NULL DEFAULT 0,
    start_date   TEXT NOT NULL,
    end_date     TEXT NOT NULL,
    status       TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','completed','expired')),
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE quest_progress (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    quest_id      INTEGER NOT NULL REFERENCES quests(id) ON DELETE CASCADE,
    current_value INTEGER NOT NULL DEFAULT 0,
    completed_at  TEXT
);

CREATE TABLE focus_sessions (
    id                       INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id                  INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    started_at               TEXT NOT NULL,
    ended_at                 TEXT,
    planned_duration_minutes INTEGER NOT NULL,
    actual_duration_seconds  INTEGER,
    status                   TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','completed','cancelled'))
);

CREATE TABLE ai_conversations (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE ai_messages (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id INTEGER NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
    role            TEXT NOT NULL CHECK (role IN ('user','assistant')),
    content         TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE app_settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE INDEX idx_tasks_status            ON tasks(status);
CREATE INDEX idx_tasks_due_at            ON tasks(due_at);
CREATE INDEX idx_tasks_category_id       ON tasks(category_id);
CREATE INDEX idx_habits_is_active        ON habits(is_active);
CREATE INDEX idx_habit_completions_date  ON habit_completions(habit_id, local_date);
CREATE INDEX idx_goal_milestones_goal_id ON goal_milestones(goal_id);
CREATE INDEX idx_scheduled_blocks_task   ON scheduled_blocks(task_id);
CREATE INDEX idx_xp_transactions_source  ON xp_transactions(source_type, source_id);
CREATE INDEX idx_quests_status           ON quests(status);
CREATE INDEX idx_focus_sessions_task_id  ON focus_sessions(task_id);
CREATE INDEX idx_ai_messages_conv        ON ai_messages(conversation_id);

INSERT INTO categories (name, color) VALUES
    ('Work',     '#06b6d4'),
    ('Health',   '#22c55e'),
    ('Personal', '#a855f7'),
    ('Learning', '#f59e0b');
