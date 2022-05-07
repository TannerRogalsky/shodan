use caith::*;

pub fn roll(input: &str) -> eyre::Result<String> {
    let roller = Roller::new(input)?;
    let result = roller.roll()?;
    Ok(result.to_string())
}
