//! Async authentication example — exits 0 on headless macOS.
//!
//! Demonstrates `AsyncLocalPlayer::authenticate()` and
//! `AsyncLocalPlayer::load_friends_authorization()`.
//!
//! Run with:
//! ```text
//! cargo run --example 15_async_auth --features async
//! ```

use gamekit::async_api::{AsyncLocalPlayer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        // Authenticate the local player.
        // On a headless/CI machine this will error ("not signed in to Game Center"),
        // so we treat any error as a graceful skip.
        match AsyncLocalPlayer::authenticate().await {
            Ok(player) => {
                println!("Authenticated as: {} ({})", player.player.alias, player.player.display_name);
            }
            Err(e) => {
                println!("authenticate skipped: {e}");
            }
        }

        // Load friends-authorization status.
        match AsyncLocalPlayer::load_friends_authorization().await {
            Ok(status) => println!("Friends auth status: {status:?}"),
            Err(e) => println!("load_friends_authorization skipped: {e}"),
        }

        Ok(())
    })
}
