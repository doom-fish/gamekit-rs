use std::error::Error;

use gamekit::{MatchRequest, MatchType, Matchmaker, MatchmakerViewController, MatchmakingMode};

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    let request = MatchRequest::default();

    if std::env::var_os(ENV_VAR).is_some() {
        let matchmaker = Matchmaker::shared();
        println!("activity: {}", matchmaker.query_activity()?);
        println!(
            "max peer players: {}",
            Matchmaker::max_players_allowed(MatchType::PeerToPeer)
        );
        println!(
            "hosted players: {:#?}",
            matchmaker.find_hosted_players(&request)?
        );
    } else {
        println!(
            "Set {ENV_VAR}=1 to query real-time matchmaking activity; default request min/max players = {}/{} and the AppKit UI uses {:?} mode via MatchmakerViewController.",
            request.min_players,
            request.max_players,
            MatchmakingMode::Default,
        );
        let _ = std::mem::size_of::<MatchmakerViewController>();
    }

    Ok(())
}
