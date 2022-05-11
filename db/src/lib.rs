pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::r2d2::ManageConnection;
use diesel::{insert_into, r2d2};

#[derive(Clone)]
pub struct DB {
    pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
}

impl DB {
    pub fn env_url() -> Result<String, std::env::VarError> {
        std::env::var("DATABASE_URL")
    }

    pub fn new<S: AsRef<str>>(database_url: S) -> eyre::Result<Self> {
        let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url.as_ref());
        manager.connect()?;
        let pool = r2d2::Pool::builder()
            .test_on_check_out(true)
            .build(manager)?;
        Ok(Self { pool })
    }

    pub fn random_jeopardy_category(
        &mut self,
    ) -> eyre::Result<(models::JeopardyCategory, Vec<models::JeopardyQuestion>)> {
        let ref mut conn = self.pool.get()?;
        let category = schema::jeopardy_categories::table
            .order(random())
            .first::<models::JeopardyCategory>(conn)?;
        let questions = models::JeopardyQuestion::belonging_to(&category)
            .load::<models::JeopardyQuestion>(conn)?;
        Ok((category, questions))
    }

    pub fn record_jeopardy_category_post(
        &mut self,
        category_id: uuid::Uuid,
        discord_message_id: u64,
    ) -> eyre::Result<usize> {
        use schema::posted_jeopardy_categories::dsl::{
            discord_message_id as dmi, id, jeopardy_category_id as jci,
        };
        let ref mut conn = self.pool.get()?;
        let inserted_count = insert_into(schema::posted_jeopardy_categories::table)
            .values((
                id.eq(uuid::Uuid::new_v4()),
                jci.eq(category_id),
                dmi.eq(discord_message_id as i64),
            ))
            .execute(conn)?;
        Ok(inserted_count)
    }

    pub fn get_jeopardy_category_post(
        &mut self,
        discord_message_id: u64,
    ) -> eyre::Result<models::PostedJeopardyCategory> {
        use schema::posted_jeopardy_categories as pjc;
        let ref mut conn = self.pool.get()?;
        let predicate = pjc::discord_message_id.eq(discord_message_id as i64);
        let result = pjc::table
            .filter(predicate)
            .first::<models::PostedJeopardyCategory>(conn)
            .optional()?;
        result
            .ok_or_else(|| eyre::eyre!("No category posted for message id: {}", discord_message_id))
    }

    pub fn increment_jeopardy_category_post(
        &mut self,
        discord_message_id: u64,
    ) -> eyre::Result<i32> {
        use schema::posted_jeopardy_categories as pjc;
        let ref mut conn = self.pool.get()?;
        let predicate = pjc::discord_message_id.eq(discord_message_id as i64);
        let result = diesel::update(pjc::table)
            .filter(predicate)
            .set(pjc::rating.eq(pjc::rating + 1))
            .returning(pjc::rating)
            .get_result::<i32>(conn)?;
        Ok(result)
    }

    pub fn decrement_jeopardy_category_post(
        &mut self,
        discord_message_id: u64,
    ) -> eyre::Result<i32> {
        use schema::posted_jeopardy_categories as pjc;
        let ref mut conn = self.pool.get()?;
        let predicate = pjc::discord_message_id.eq(discord_message_id as i64);
        let result = diesel::update(pjc::table)
            .filter(predicate)
            .set(pjc::rating.eq(pjc::rating - 1))
            .returning(pjc::rating)
            .get_result::<i32>(conn)?;
        Ok(result)
    }
}

sql_function!(fn random() -> Text);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_category_test() {
        dotenv::dotenv().unwrap();
        let mut db = DB::new(DB::env_url().unwrap()).unwrap();
        let (category, _questions) = db.random_jeopardy_category().unwrap();
        let count = db.record_jeopardy_category_post(category.id, 0).unwrap();
        assert_eq!(count, 1);
    }
}
