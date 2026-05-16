use std::error::Error;

use gamekit::{GameActivity, GameActivityDefinition};

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os(ENV_VAR).is_some() {
        println!(
            "pending game activities: {}",
            GameActivity::has_pending_game_activities()?
        );
        let definitions = GameActivityDefinition::load_all()?;
        println!("definitions: {definitions:#?}");
        if let Some(definition) = definitions.first() {
            let activity = GameActivity::new(definition)?;
            println!("activity snapshot: {:#?}", activity.snapshot()?);
        }
    } else {
        println!(
            "Set {ENV_VAR}=1 to load macOS 26 GameActivity definitions and pending-activity state."
        );
    }

    Ok(())
}
