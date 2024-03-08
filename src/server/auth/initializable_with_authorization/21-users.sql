CREATE TABLE IF NOT EXISTS users
(
	id uuid PRIMARY KEY,
	employee_id uuid REFERENCES employees(id),
	password text NOT NULL,
	password_set timestamp NOT NULL,
	role_id uuid NOT NULL REFERENCES roles(id),
	username text NOT NULL UNIQUE
);
