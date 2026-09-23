use std::time::{Duration, Instant};

use gamekit::{GameKitError, Leaderboard, LocalPlayer};

fn main() {
    let started = Instant::now();

    let recent = LocalPlayer::load_recent_players();
    assert!(
        matches!(recent, Err(GameKitError::WrongThread(_))),
        "recent players on the main thread: {recent:?}"
    );
    let leaderboards = Leaderboard::load_all();
    assert!(
        matches!(leaderboards, Err(GameKitError::WrongThread(_))),
        "leaderboards on the main thread: {leaderboards:?}"
    );

    assert!(started.elapsed() < Duration::from_secs(5));
    println!("test synchronous_calls_on_the_main_thread_fail_fast ... ok");
}
