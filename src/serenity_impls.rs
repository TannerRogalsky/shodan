pub struct DalleSupport;

impl serenity::prelude::TypeMapKey for DalleSupport {
    type Value = Option<dalle::Dalle>;
}

pub struct DoSpacesSupport;

impl serenity::prelude::TypeMapKey for DoSpacesSupport {
    type Value = do_spaces::Client;
}
