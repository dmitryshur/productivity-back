SET timezone = 'Europe/Moscow';
CREATE EXTENSION pgcrypto;

CREATE TABLE todo(
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL,
    title VARCHAR(50) NOT NULL,
    body TEXT,
    creation_date TIMESTAMPTZ NOT NULL,
    last_edit_date TIMESTAMPTZ NOT NULL,
    done BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE account(
    id SERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL
);

GRANT ALL PRIVILEGES ON TABLE todo TO dshur;
GRANT ALL PRIVILEGES ON TABLE account TO dshur;
GRANT ALL PRIVILEGES ON SEQUENCE todo_id_seq TO dshur;
GRANT ALL PRIVILEGES ON SEQUENCE account_id_seq TO dshur;
