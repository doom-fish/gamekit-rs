use std::error::Error;

use gamekit::AccessPoint;

const ENV_VAR: &str = "GAMEKIT_RS_RUN_LIVE";

fn main() -> Result<(), Box<dyn Error>> {
    let access_point = AccessPoint::shared();

    if std::env::var_os(ENV_VAR).is_some() {
        let snapshot = access_point.snapshot()?;
        println!("access point snapshot: {snapshot:#?}");
    } else {
        println!("Set {ENV_VAR}=1 to call AccessPoint::shared().snapshot().");
    }

    Ok(())
}
