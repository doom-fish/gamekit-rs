# gamekit-rs

Safe Rust bindings for Apple's `GameKit` framework on macOS.

## What is covered in v0.2.2

`gamekit-rs` now reaches 100% of the audited top-level macOS-available `GameKit` surface in [`COVERAGE_AUDIT.md`](COVERAGE_AUDIT.md) and exposes logical modules for:

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
- `AccessPoint`
- `ChallengeDefinition`
- `Score`
- `SavedGame`

See [`COVERAGE.md`](COVERAGE.md) for the audited SDK coverage table, deprecated Apple APIs that are still exposed, runtime gating notes, and the remaining member-level omissions.

## Requirements

- macOS 12.0+
- Xcode / Swift toolchain
- macOS 26 SDK if you want `ChallengeDefinition` and `GameActivity` support

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
