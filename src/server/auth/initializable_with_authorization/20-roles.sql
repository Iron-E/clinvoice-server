CREATE TABLE IF NOT EXISTS roles
(
	id uuid PRIMARY KEY,
	name text NOT NULL UNIQUE,
	password_ttl interval
);
