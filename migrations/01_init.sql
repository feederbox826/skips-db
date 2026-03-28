CREATE TABLE users (
  user_id TEXT PRIMARY KEY,
  name TEXT DEFAULT NULL
);

CREATE TABLE submissions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id TEXT NOT NULL,
  studio_id TEXT NOT NULL,
  skip_seconds REAL NOT NULL DEFAULT 0,
  created_at TEXT DEFAULT current_date,
  FOREIGN KEY (user_id) REFERENCES users(user_id)
);
CREATE UNIQUE INDEX idx_submissions_user ON submissions(studio_id, user_id);
CREATE UNIQUE INDEX idx_submissions_start ON submissions(studiO_id, skip_seconds);

CREATE TABLE votes (
  submission_id INTEGER NOT NULL,
  user_id TEXT NOT NULL,
  vote INTEGER NOT NULL,
  PRIMARY KEY (submission_id, user_id),
  FOREIGN KEY (submission_id) REFERENCES submissions(id),
  FOREIGN KEY (user_id) REFERENCES users(user_id)
);
CREATE INDEX idx_votes_submission ON votes(submission_id);

CREATE TABLE studio_aggregates (
  studio_id TEXT PRIMARY KEY NOT NULL,
  skip_seconds REAL NOT NULL
);
