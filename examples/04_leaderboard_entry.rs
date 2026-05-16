use std::error::Error;

use gamekit::{Leaderboard, PlayerScope, TimeScope};

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os(ENV_VAR).is_some() {
        let leaderboards = Leaderboard::load_all()?;
        if let Some(leaderboard) = leaderboards.first() {
            let entries = leaderboard.load_entries(PlayerScope::Global, TimeScope::AllTime, 0..10)?;
            println!("entries: {entries:#?}");
        } else {
            println!("No leaderboards were returned for the current game.");
        }
    } else {
        println!(
            "Set {ENV_VAR}=1 to call Leaderboard::load_all() and load_entries(...)."
        );
    }

    Ok(())
}
