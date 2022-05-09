use db::models::{JeopardyCategory, JeopardyQuestion};

pub fn print(category: &JeopardyCategory, questions: &[JeopardyQuestion]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(
        &mut out,
        "{} - #{} - Category: ||{}||",
        category.round, category.jeopardy_show_number, category.name
    )
    .unwrap();
    for question in questions {
        let value = question
            .value
            .map(|v| format!("${}", v).into())
            .unwrap_or(std::borrow::Cow::Borrowed("Unknown"));
        writeln!(
            &mut out,
            "{} - {} - ||{}||",
            value, question.question, question.answer
        )
        .unwrap();
    }

    out
}
