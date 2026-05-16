use std::error::Error;

use gamekit::Leaderboard;

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";
const LEADERBOARD_IDS: [&str; 1] = ["com.example.high-score"];

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os(ENV_VAR).is_some() {
        let leaderboards = Leaderboard::load(&LEADERBOARD_IDS)?;
        println!("leaderboards: {leaderboards:#?}");
    } else {
        println!(
            "Set {ENV_VAR}=1 to call Leaderboard::load(&{LEADERBOARD_IDS:?})."
        );
    }

    Ok(())
}
