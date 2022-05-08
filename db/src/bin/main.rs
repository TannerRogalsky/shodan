use db::*;
use diesel::prelude::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().unwrap();

    let uri = std::env::var("DATABASE_URL").unwrap();
    let mut connection = diesel::pg::PgConnection::establish(&uri).unwrap();

    {
        let query = schema::jeopardy_shows::table.select(schema::jeopardy_shows::all_columns);
        println!("{}", diesel::debug_query(&query));
        let results = query.load::<models::JeopardyShow>(&mut connection).unwrap();
        println!("{:#?}", results);
    }

    {
        let query = schema::jeopardy_shows::table
            .inner_join(schema::jeopardy_categories::table)
            .select((
                schema::jeopardy_shows::all_columns,
                schema::jeopardy_categories::all_columns,
            ));
        println!("{}", diesel::debug_query(&query));
        let results = query
            .load::<(models::JeopardyShow, models::JeopardyCategory)>(&mut connection)
            .unwrap();
        println!("{:?}", results);
    }

    Ok(())
}
