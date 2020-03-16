SET timezone = 'Europe/Moscow';
CREATE EXTENSION pgcrypto;

CREATE TABLE todo(
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    title VARCHAR(50) NOT NULL,
    body TEXT,
    creation_date TIMESTAMPTZ NOT NULL,
    last_edit_date TIMESTAMPTZ NOT NULL,
    done BOOLEAN NOT NULL DEFAULT false
);

GRANT ALL PRIVILEGES ON TABLE todo TO dshur;
GRANT ALL PRIVILEGES ON SEQUENCE todo_id_seq TO dshur;
