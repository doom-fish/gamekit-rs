# Changelog

## [0.2.0] - 2026-05-16

### Added
- New Rust modules and Swift bridge files for `AccessPoint`, `ChallengeDefinition`, `LeaderboardEntry`, `NotificationBanner`, `Score`, `SavedGame`, `Match`, `RealTime`, and `TurnBased`
- Expanded `LocalPlayer`, `Player`, `Leaderboard`, and `Achievement` coverage
- One runnable example per logical area under `examples/`
- Area smoke tests under `tests/area_smoke.rs`
- `COVERAGE.md` with an audited GameKit surface summary and documented gaps

### Changed
- Refactored the bridge into a per-area Swift/Rust layout modeled after `screencapturekit-rs`
- Updated `build.rs` to detect SDK version, forward newer-SDK defines, and build the Swift package in the same style as the reference crate
- Added `GameKitError::Unavailable` for SDK/OS-gated APIs such as `ChallengeDefinition`
- Updated crate metadata, README, and package contents for the `0.2.0` release

## [0.1.0] - 2024-01-01

### Added
- `LocalPlayer::local()` — snapshot of the authenticated local player
- `AuthObserver` — authenticate-handler guard with Rust closure callback
- `Leaderboard::load()`, `submit_score()`, `load_entries()`
- `Achievement::report()`, `Achievement::report_all()`
- `Matchmaker::shared()`, `find_match()`, `cancel()`
- `Match` with `connected_players()`, `send_data()`, `set_delegate()`
- `Player` struct with `game_player_id`, `alias`, `display_name`
