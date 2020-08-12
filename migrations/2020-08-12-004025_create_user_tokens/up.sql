CREATE TABLE user_tokens (
  id SERIAL PRIMARY KEY,
  token TEXT NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users (id),
  service TEXT NOT NULL
)
