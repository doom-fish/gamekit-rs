use std::error::Error;

use gamekit::{MatchmakingMode, TurnBasedMatch, TurnBasedMatchmakerViewController};

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os(ENV_VAR).is_some() {
        let matches = TurnBasedMatch::load_matches()?;
        println!("turn-based matches: {matches:#?}");
    } else {
        println!(
            "Set {ENV_VAR}=1 to call TurnBasedMatch::load_matches(); TurnBasedMatchmakerViewController supports {:?} mode in the AppKit UI.",
            MatchmakingMode::InviteOnly,
        );
        let _ = std::mem::size_of::<TurnBasedMatchmakerViewController>();
    }

    Ok(())
}
