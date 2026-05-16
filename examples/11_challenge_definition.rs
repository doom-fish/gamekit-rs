use std::error::Error;

use gamekit::ChallengeDefinition;

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os(ENV_VAR).is_some() {
        let definitions = ChallengeDefinition::load_all()?;
        println!("challenge definitions: {definitions:#?}");
    } else {
        println!("Set {ENV_VAR}=1 to call ChallengeDefinition::load_all().");
    }

    Ok(())
}
