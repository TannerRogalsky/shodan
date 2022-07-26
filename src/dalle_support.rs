pub struct DalleSupport;

impl serenity::prelude::TypeMapKey for DalleSupport {
    type Value = Option<dalle::Dalle>;
}
