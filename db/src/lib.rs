pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::ManageConnection;

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
}

sql_function!(fn random() -> Text);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
