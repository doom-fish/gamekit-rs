use std::error::Error;

use gamekit::{MatchRequest, Matchmaker};

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    let request = MatchRequest::default();

    if std::env::var_os(ENV_VAR).is_some() {
        let match_handle = Matchmaker::shared().find_match(&request)?;
        println!("players: {:#?}", match_handle.players()?);
        println!(
            "expected player count: {}",
            match_handle.expected_player_count()
        );
    } else {
        println!(
            "Set {ENV_VAR}=1 to create a match with Matchmaker::find_match(); current default min/max players = {}/{}.",
            request.min_players,
            request.max_players,
        );
    }

    Ok(())
}
