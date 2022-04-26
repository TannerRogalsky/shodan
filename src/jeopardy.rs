use rand::prelude::*;

#[derive(Debug, serde::Deserialize)]
pub struct Record {
    #[serde(rename = "Show Number")]
    pub show_number: u32,
    #[serde(rename = "Air Date")]
    pub air_date: time::Date,
    // round: String,
    #[serde(rename = "Category")]
    pub category: String,
    #[serde(rename = "Value")]
    pub value: String,
    #[serde(rename = "Question")]
    pub question: String,
    #[serde(rename = "Answer")]
    pub answer: String,
}

pub struct Jeopardy {
    shows: Vec<u32>,
    by_show: std::collections::HashMap<u32, Vec<Record>>,
}

impl serenity::prelude::TypeMapKey for Jeopardy {
    type Value = Jeopardy;
}

impl Jeopardy {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("JEOPARDY.csv");

        let reader = csv::Reader::from_path(path)?;
        let parsed = reader
            .into_deserialize()
            .collect::<Result<Vec<Record>, _>>()?;

        let mut by_show = std::collections::HashMap::<u32, Vec<Record>>::new();
        for record in parsed {
            let entry = by_show.entry(record.show_number).or_default();
            entry.push(record);
        }
        let shows = by_show.keys().copied().collect::<Vec<_>>();
        Ok(Self { shows, by_show })
    }

    pub fn random(&self) -> Result<Vec<&Record>, &'static str> {
        let rng = &mut thread_rng();
        match self.shows.choose(rng) {
            None => Err("No shows loaded!"),
            Some(show_number) => {
                let show = &self.by_show[show_number];
                let mut by_category = std::collections::HashMap::<&str, Vec<&Record>>::new();
                for record in show {
                    let entry = by_category.entry(record.category.as_str()).or_default();
                    entry.push(record);
                }
                let category = by_category
                    .drain()
                    .filter(|(_, cat)| cat.len() == 5)
                    .choose(rng);
                match category {
                    None => Err("No 5 question categories!"),
                    Some((_title, category)) => Ok(category),
                }
            }
        }
    }

    pub fn fmt_category(category: &Vec<&Record>) -> String {
        use std::fmt::Write;
        let mut out = String::new();

        match category.first() {
            None => out,
            Some(first) => {
                writeln!(
                    &mut out,
                    "{:?} - #{} - Category: ||{}||",
                    first.air_date.to_calendar_date(),
                    first.show_number,
                    first.category
                )
                .unwrap();
                for question in category {
                    writeln!(&mut out, "{} - {}", question.value, question.question).unwrap();
                }

                out
            }
        }
    }
}
