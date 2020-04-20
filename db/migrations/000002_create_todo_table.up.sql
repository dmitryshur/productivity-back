CREATE TABLE todo(
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL,
    title VARCHAR(50) NOT NULL,
    body TEXT,
    creation_date TIMESTAMPTZ NOT NULL,
    last_edit_date TIMESTAMPTZ NOT NULL,
    done BOOLEAN NOT NULL DEFAULT false
);
