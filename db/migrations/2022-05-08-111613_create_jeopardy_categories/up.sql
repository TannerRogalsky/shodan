CREATE TABLE jeopardy_categories (
    id UUID PRIMARY KEY,
    jeopardy_show_number INT NOT NULL REFERENCES jeopardy_shows(show_number),
    name TEXT NOT NULL,
    round TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('jeopardy_categories');