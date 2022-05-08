use db::models::{NewJeopardyCategory, NewJeopardyShow};
use diesel::prelude::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().unwrap();

    let uri = std::env::var("DATABASE_URL").unwrap();
    let mut connection = diesel::pg::PgConnection::establish(&uri).unwrap();

    jeopardy_seed(&mut connection);

    Ok(())
}

fn jeopardy_seed(conn: &mut PgConnection) {
    let j = jeopardy::Jeopardy::new().unwrap();
    let shows = j
        .shows()
        .iter()
        .map(|show_number| {
            let air_date = j
                .categories(show_number)
                .unwrap()
                .first()
                .map(|record| record.air_date)
                .unwrap();
            let show_number = *show_number as i32;
            let (year, ordinal) = air_date.to_ordinal_date();
            NewJeopardyShow {
                show_number,
                air_date: chrono::NaiveDate::from_yo(year as _, ordinal as _),
            }
        })
        .collect::<Vec<_>>();

    let mut categories = vec![];
    let mut questions = vec![];
    for show_number in j.shows() {
        let mut seen = std::collections::HashMap::new();
        for question in j.categories(show_number).unwrap() {
            let jeopardy_category_id = match seen.get(question.category.as_str()).copied() {
                Some(jeopardy_category_id) => jeopardy_category_id,
                None => {
                    let id = uuid::Uuid::new_v4();
                    seen.insert(question.category.as_str(), id);
                    categories.push(NewJeopardyCategory {
                        id,
                        jeopardy_show_number: *show_number as _,
                        name: question.category.as_str(),
                        round: question.round.as_str(),
                    });
                    id
                }
            };
            questions.push({
                // mostly this fails on "None"
                let value = question
                    .value
                    .as_str()
                    .trim()
                    .replace('$', "")
                    .replace(',', "")
                    .parse()
                    .ok();
                db::models::NewJeopardyQuestion {
                    id: uuid::Uuid::new_v4(),
                    jeopardy_category_id,
                    value,
                    question: question.question.as_str(),
                    answer: question.answer.as_str(),
                }
            });
        }
    }

    diesel::insert_into(db::schema::jeopardy_shows::dsl::jeopardy_shows)
        .values(&shows)
        .execute(conn)
        .unwrap();
    for (index, category_chunk) in categories.chunks(10000).enumerate() {
        diesel::insert_into(db::schema::jeopardy_categories::dsl::jeopardy_categories)
            .values(category_chunk)
            .execute(conn)
            .unwrap();
        println!(
            "Seed category cluster {} / {}.",
            index,
            categories.len() / 10000
        );
    }
    for (index, questions_chunk) in questions.chunks(10000).enumerate() {
        diesel::insert_into(db::schema::jeopardy_questions::dsl::jeopardy_questions)
            .values(questions_chunk)
            .execute(conn)
            .unwrap();
        println!(
            "Seed questions cluster {} / {}.",
            index,
            questions.len() / 10000
        );
    }
}
