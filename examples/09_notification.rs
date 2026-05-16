use std::error::Error;

use gamekit::NotificationBanner;

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os(ENV_VAR).is_some() {
        NotificationBanner::show(Some("gamekit-rs"), Some("Hello from GameKit"))?;
        println!("Displayed a GameKit notification banner.");
    } else {
        println!("Set {ENV_VAR}=1 to display a GameKit notification banner.");
    }

    Ok(())
}
