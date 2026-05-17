//! Async leaderboard example — exits 0 on headless macOS.
//!
//! Demonstrates `AsyncLeaderboard::load()` and `AsyncLeaderboard::load_entries()`.
//!
//! Run with:
//! ```text
//! cargo run --example 16_async_leaderboard --features async
//! ```

use gamekit::async_api::AsyncLeaderboard;
use gamekit::leaderboard::{PlayerScope, TimeScope};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        // Load all leaderboards (empty slice = all).
        match AsyncLeaderboard::load(&[])?.await {
            Ok(leaderboards) => {
                println!("Loaded {} leaderboard(s)", leaderboards.len());
                for lb in &leaderboards {
                    println!("  {} — {:?}", lb.base_leaderboard_id, lb.leaderboard_type);
                }

                // Load entries for the first leaderboard if any exist.
                if let Some(lb) = leaderboards.first() {
                    match AsyncLeaderboard::load_entries(
                        &lb.base_leaderboard_id,
                        PlayerScope::Global,
                        TimeScope::AllTime,
                        1..11,
                    )?
                    .await
                    {
                        Ok(result) => {
                            println!(
                                "Top {} entries (total {})",
                                result.entries.len(),
                                result.total_player_count
                            );
                        }
                        Err(e) => println!("load_entries skipped: {e}"),
                    }
                }
            }
            Err(e) => println!("load leaderboards skipped: {e}"),
        }

        Ok(())
    })
}
