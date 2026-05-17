//! Async saved-game example — exits 0 on headless macOS.
//!
//! Demonstrates `AsyncSavedGame::fetch_all()`, `AsyncSavedGame::load_data()`,
//! and `AsyncSavedGame::save()`.
//!
//! Run with:
//! ```text
//! cargo run --example 18_async_saved_game --features async
//! ```

use gamekit::async_api::AsyncSavedGame;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        // Fetch all saved games.
        match AsyncSavedGame::fetch_all().await {
            Ok(games) => {
                println!("Fetched {} saved game(s)", games.len());
                for g in &games {
                    println!(
                        "  name={:?} device={:?} date={:?}",
                        g.name, g.device_name, g.modification_date
                    );
                }

                // Load data for the first saved game (if any).
                if let Some(first) = games.first() {
                    match AsyncSavedGame::load_data(first)?.await {
                        Ok(data) => println!("Loaded {} bytes of saved-game data", data.len()),
                        Err(e) => println!("load_data skipped: {e}"),
                    }
                }
            }
            Err(e) => println!("fetch_all skipped: {e}"),
        }

        // Save a tiny payload.  Errors are graceful skips on headless machines.
        match AsyncSavedGame::save("async_example_slot", b"hello world")?.await {
            Ok(saved) => println!("Saved game: {:?}", saved.name),
            Err(e) => println!("save skipped: {e}"),
        }

        Ok(())
    })
}
