// @generated automatically by Diesel CLI.

diesel::table! {
    jeopardy_categories (id) {
        id -> Uuid,
        jeopardy_show_number -> Int4,
        name -> Text,
        round -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    jeopardy_questions (id) {
        id -> Uuid,
        jeopardy_category_id -> Uuid,
        value -> Nullable<Int4>,
        question -> Text,
        answer -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    jeopardy_shows (show_number) {
        show_number -> Int4,
        air_date -> Date,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(jeopardy_categories -> jeopardy_shows (jeopardy_show_number));
diesel::joinable!(jeopardy_questions -> jeopardy_categories (jeopardy_category_id));

diesel::allow_tables_to_appear_in_same_query!(
    jeopardy_categories,
    jeopardy_questions,
    jeopardy_shows,
);
