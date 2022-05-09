pub struct DB;

impl serenity::prelude::TypeMapKey for DB {
    type Value = db::DB;
}
