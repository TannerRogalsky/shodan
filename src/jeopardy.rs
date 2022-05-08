pub struct Jeopardy(pub jeopardy::Jeopardy);

impl serenity::prelude::TypeMapKey for Jeopardy {
    type Value = jeopardy::Jeopardy;
}
