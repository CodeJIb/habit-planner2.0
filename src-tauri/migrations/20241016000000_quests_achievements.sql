-- Phase 3: quests and achievements.
-- Quest rows are generated for the current day/week from templates in services/quest.rs.
-- `code` + `period_key` identify a template for one period (e.g. daily:2026-10-16).
-- `metric` names the counters a quest tracks (tasks_completed, habits_completed).

ALTER TABLE quests ADD COLUMN code        TEXT;
ALTER TABLE quests ADD COLUMN metric      TEXT NOT NULL DEFAULT 'tasks_completed';
ALTER TABLE quests ADD COLUMN period_key  TEXT;
ALTER TABLE quests ADD COLUMN completed_at TEXT;
ALTER TABLE quests ADD COLUMN xp_rewarded INTEGER NOT NULL DEFAULT 0;

-- One row per template per period; NULLs (hand-made quests) are unaffected.
CREATE UNIQUE INDEX idx_quests_code_period ON quests(code, period_key);

-- Achievement definitions. INSERT OR IGNORE keeps this safe if it ever re-runs.
INSERT OR IGNORE INTO achievements (code, name, description, icon, xp_reward) VALUES
    ('first_task',    'First Steps',      'Complete your first task.',             'check', 10),
    ('task_master_10','Getting Things Done','Complete 10 tasks.',                   'check', 30),
    ('task_master_50','Task Machine',     'Complete 50 tasks.',                    'check', 100),
    ('first_habit',   'Habit Starter',    'Log your first habit check-in.',        'flame', 10),
    ('streak_7',      'Week Warrior',     'Reach a 7-day streak on any habit.',    'flame', 50),
    ('level_5',       'Rising Star',      'Reach player level 5.',                 'star',  75);
