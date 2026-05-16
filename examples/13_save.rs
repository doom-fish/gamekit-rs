use std::error::Error;

use gamekit::SavedGame;

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os(ENV_VAR).is_some() {
        let saved_games = SavedGame::fetch_all()?;
        println!("saved games: {saved_games:#?}");
    } else {
        println!("Set {ENV_VAR}=1 to call SavedGame::fetch_all().");
    }

    Ok(())
}
