CREATE TABLE IF NOT EXISTS sessions
(
	id text NOT NULL PRIMARY KEY,
	expiry timestamp,
	session json NOT NULL
);
