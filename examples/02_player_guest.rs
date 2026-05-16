use std::error::Error;

use gamekit::Player;

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os(ENV_VAR).is_some() {
        let guest = Player::anonymous_guest("guest-demo")?;
        println!("guest player: {guest:#?}");
    } else {
        println!("Set {ENV_VAR}=1 to create a GameKit anonymous guest snapshot.");
    }

    Ok(())
}
