-- Habit reminders: remember the date a habit's reminder was last pushed so it
-- fires once per day, the same way tasks.reminded_at works.
ALTER TABLE habits ADD COLUMN reminded_on TEXT;
