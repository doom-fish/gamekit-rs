//! Async achievement example — exits 0 on headless macOS.
//!
//! Demonstrates `AsyncAchievement::load()` and `AsyncAchievement::report()`.
//!
//! Run with:
//! ```text
//! cargo run --example 17_async_achievement --features async
//! ```

use gamekit::async_api::AsyncAchievement;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        // Load existing achievements.
        match AsyncAchievement::load().await {
            Ok(achievements) => {
                println!("Loaded {} achievement(s)", achievements.len());
                for ach in &achievements {
                    println!(
                        "  {} — {:.0}% complete",
                        ach.identifier, ach.percent_complete
                    );
                }

                // Re-report the same achievements (no-op if already complete).
                // Errors are treated as graceful skips on headless machines.
                if !achievements.is_empty() {
                    match AsyncAchievement::report(&achievements)?.await {
                        Ok(()) => println!("Achievements reported successfully"),
                        Err(e) => println!("report skipped: {e}"),
                    }
                }
            }
            Err(e) => println!("load achievements skipped: {e}"),
        }

        Ok(())
    })
}
