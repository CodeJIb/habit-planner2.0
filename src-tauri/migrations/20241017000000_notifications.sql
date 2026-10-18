-- Phone reminders: remember when a task's due alert was pushed so it is sent
-- once and not repeated on every scheduler tick.
ALTER TABLE tasks ADD COLUMN reminded_at TEXT;
