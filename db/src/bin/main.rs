use db::*;
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().unwrap();

    let mut connection = DB::new(DB::env_url().unwrap()).unwrap();

    {
        let (cat, questions) = connection.random_jeopardy_category().unwrap();
        println!("{:#?}", cat);
        println!("{:#?}", questions);
    }

    // {
    // use diesel::prelude::*;
    // use db::schema::jeopardy_questions::question;
    //     let query = schema::jeopardy_shows::table
    //         .inner_join(schema::jeopardy_categories::table)
    //         .select((
    //             schema::jeopardy_shows::all_columns,
    //             schema::jeopardy_categories::all_columns,
    //         ));
    //     println!("{}", diesel::debug_query(&query));
    //     let results = query
    //         .load::<(models::JeopardyShow, models::JeopardyCategory)>(&mut connection)
    //         .unwrap();
    //     println!("{:?}", results);
    // }

    Ok(())
}
