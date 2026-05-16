use std::error::Error;

use gamekit::Score;

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    let scores = [Score::new_local("com.example.high-score", 1_000, 0)];

    if std::env::var_os(ENV_VAR).is_some() {
        Score::report_all(&scores)?;
        println!("Reported scores: {scores:#?}");
    } else {
        println!("Set {ENV_VAR}=1 to call Score::report_all(&scores).");
    }

    Ok(())
}
