use std::error::Error;

use gamekit::Achievement;

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os(ENV_VAR).is_some() {
        let achievements = Achievement::load()?;
        let descriptions = Achievement::load_descriptions()?;
        println!("achievements: {achievements:#?}");
        println!("descriptions: {descriptions:#?}");
    } else {
        println!(
            "Set {ENV_VAR}=1 to call Achievement::load() and Achievement::load_descriptions()."
        );
    }

    Ok(())
}
