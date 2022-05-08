CREATE TABLE jeopardy_questions (
    id UUID PRIMARY KEY,
    jeopardy_category_id UUID NOT NULL REFERENCES jeopardy_categories(id),
    value INT,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('jeopardy_questions');