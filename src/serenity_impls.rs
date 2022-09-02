pub struct DoSpacesSupport;

impl serenity::prelude::TypeMapKey for DoSpacesSupport {
    type Value = do_spaces::Client;
}

pub struct StableDiffusionSupport;

impl serenity::prelude::TypeMapKey for StableDiffusionSupport {
    type Value = stable_diffusion::Client;
}
