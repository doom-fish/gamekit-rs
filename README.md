# gamekit-rs

Safe Rust bindings for Apple's `GameKit` framework on macOS.

## What is covered in v0.8.9

`gamekit-rs` covers 65 of the 67 audited top-level macOS-available `GameKit` symbols in [`COVERAGE_AUDIT.md`](COVERAGE_AUDIT.md) (the two gaps are enums used only by the deprecated `GKGameSession` API) and exposes logical modules for:

- `BasePlayer`, `Player`, and `LocalPlayer`
- `LocalPlayerListener`
- typed framework errors and exported `GameKit` constants
- `Leaderboard`, `LeaderboardSet`, and `LeaderboardEntry`
- `Achievement`
- `Match`
- `TurnBased`
- `RealTime` matchmaking, including `MatchedPlayers` and invite-recipient responses
- `MatchmakerViewController` / `TurnBasedMatchmakerViewController`
- `DialogController`, including legacy Game Center dismissal callbacks
- `GameActivity`
- notification banners
- `AccessPoint`, including every `GKAccessPoint` trigger variant through `AccessPointTrigger`
- `ChallengeDefinition`
- `Score`
- `SavedGame`

### Async API (feature = `"async"`)

Enable the `async` feature to get executor-agnostic `Future` wrappers for the currently wrapped `GameKit` completion-handler families:

```toml
[dependencies]
gamekit = { version = "0.8.9", features = ["async"] }
```

| Wrapper | Futures |
|---|---|
| `AsyncLocalPlayer` | `authenticate`, `friends_authorization_status` |
| `AsyncMatchmaker` | `find_match`, `find_players` |
| `AsyncLeaderboard` | `load_leaderboards`, `load_entries` |
| `AsyncAchievement` | `load_achievements`, `load_descriptions`, `report_achievement`, `reset` |
| `AsyncLeaderboardSet` | `load_leaderboards`, `load_image_data` |
| `AsyncChallengeDefinition` | `load_image_data` |
| `AsyncSavedGame` | `fetch_all_saved_games`, `load_data`, `save_game` |

```rust,no_run
#[cfg(feature = "async")]
async fn example() -> Result<(), Box<dyn std::error::Error>> {
    use gamekit::async_api::AsyncLocalPlayer;
    let player = AsyncLocalPlayer::authenticate().await?;
    println!("authenticated as {}", player.player.display_name);
    Ok(())
}
```

See [`COVERAGE.md`](COVERAGE.md) for the audited SDK coverage table, deprecated Apple APIs that are still exposed, runtime gating notes, and the remaining member-level omissions.

## Requirements

- macOS 12.0+
- Xcode / Swift toolchain
- macOS 26 SDK if you want `ChallengeDefinition` and `GameActivity` support

The Swift bridge is built for macOS 12.0 even with the macOS 26 SDK. APIs from newer releases check the running system and return `GameKitError::Unavailable` on older versions.

## Threading

- Synchronous calls wait for `GameKit`, which delivers its answers on the main thread. On the main thread they return `GameKitError::WrongThread` instead of deadlocking, so call them from another thread or use the `async` feature.
- `MatchmakerViewController`, `TurnBasedMatchmakerViewController`, `SignInViewController` and `DialogController` presentation are `AppKit` UI. Create, present and dismiss them on the main thread; the view controllers are not `Send`, and calls from other threads return `GameKitError::WrongThread`.
- `GameKit` cannot look up recipients while a matchmaking view controller is created on the main thread, so a request with `recipient_ids` returns `GameKitError::Unavailable`. Invite players from the controller's UI, or use `Matchmaker` from another thread.
- `AccessPoint::trigger`, `trigger_state` and `trigger_target` return immediately. Pass a completion to `trigger_target` to learn when Game Center closes.
- When `GameKit` asks the player to sign in, `LocalPlayer::sign_in_view_controller()` returns the view controller it provided, ready to present on the main thread.
- Friend and player lookups need a bundle identifier. `GameKit` raises an exception without one, so these calls return `GameKitError::Unavailable` instead.

## Usage

```rust,no_run
use gamekit::{ErrorCode, LocalPlayer, ERROR_DOMAIN};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = LocalPlayer::local()?;
    println!("authenticated: {}", player.is_authenticated);

    let framework_error = gamekit::GameKitFrameworkError {
        domain: ERROR_DOMAIN.to_owned(),
        code: 33,
        localized_description: "connection timed out".to_owned(),
    };
    assert_eq!(framework_error.error_code(), Some(ErrorCode::ConnectionTimeout));
    Ok(())
}
```

## Examples

Each logical area has an example under `examples/`.

```sh
cargo run --example 01_gamekit_smoke
cargo run --example 08_real_time
cargo run --example 14_game_activity
```

By default, the examples run in dry-run mode so they are safe in CI and on machines that are not signed into Game Center. Set `GAMEKIT_RS_RUN_LIVE=1` to execute the live `GameKit` calls.

## License

MIT OR Apache-2.0
