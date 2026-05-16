# gamekit-rs

Safe Rust bindings for Apple's `GameKit` framework on macOS.

## What is covered in v0.2.1

`gamekit-rs` now exposes logical modules for:

- `LocalPlayer`
- `LocalPlayerListener`
- `Player`
- `Leaderboard`
- `LeaderboardEntry`
- `Achievement`
- `Match`
- `TurnBased`
- `RealTime` matchmaking
- `MatchmakerViewController` / `TurnBasedMatchmakerViewController`
- `DialogController`
- `GameActivity`
- notification banners
- `AccessPoint`
- `ChallengeDefinition`
- `Score`
- `SavedGame`

See [`COVERAGE.md`](COVERAGE.md) for the audited SDK coverage table, deprecated Apple APIs that are still exposed, and the remaining known gaps.

## Requirements

- macOS 12.0+
- Xcode / Swift toolchain
- macOS 26 SDK if you want `ChallengeDefinition` and `GameActivity` support

## Usage

```rust,no_run
use gamekit::LocalPlayer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = LocalPlayer::local()?;
    println!("authenticated: {}", player.is_authenticated);
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
