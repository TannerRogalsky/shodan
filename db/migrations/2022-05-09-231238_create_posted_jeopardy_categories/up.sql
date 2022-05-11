CREATE TABLE posted_jeopardy_categories (
    id UUID PRIMARY KEY,
    jeopardy_category_id UUID NOT NULL REFERENCES jeopardy_categories(id),
    discord_message_id BIGINT NOT NULL,
    rating INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('posted_jeopardy_categories');