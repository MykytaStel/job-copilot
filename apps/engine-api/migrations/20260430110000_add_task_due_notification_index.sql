CREATE INDEX IF NOT EXISTS tasks_pending_remind_at_idx
    ON tasks (remind_at, application_id)
    WHERE done = FALSE AND remind_at IS NOT NULL;
