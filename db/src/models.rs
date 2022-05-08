use crate::schema::*;
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Identifiable)]
#[diesel(primary_key(show_number))]
pub struct JeopardyShow {
    pub show_number: i32,
    pub air_date: chrono::NaiveDate,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(JeopardyShow, foreign_key = jeopardy_show_number), table_name = jeopardy_categories)]
pub struct JeopardyCategory {
    pub id: uuid::Uuid,
    pub jeopardy_show_number: i32,
    pub name: String,
    pub round: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(JeopardyCategory))]
pub struct JeopardyQuestion {
    pub id: uuid::Uuid,
    pub jeopardy_category_id: uuid::Uuid,
    pub value: Option<i32>,
    pub question: String,
    pub answer: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = jeopardy_shows)]
pub struct NewJeopardyShow {
    pub show_number: i32,
    pub air_date: chrono::NaiveDate,
}

#[derive(Insertable)]
#[diesel(table_name = jeopardy_categories)]
pub struct NewJeopardyCategory<'a> {
    pub id: uuid::Uuid,
    pub jeopardy_show_number: i32,
    pub name: &'a str,
    pub round: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = jeopardy_questions)]
pub struct NewJeopardyQuestion<'a> {
    pub id: uuid::Uuid,
    pub jeopardy_category_id: uuid::Uuid,
    pub value: Option<i32>,
    pub question: &'a str,
    pub answer: &'a str,
}
