# Changelog

## [0.2.2] - 2026-05-17

### Added
- `BasePlayer`, `LeaderboardSet`, `MatchedPlayers`, `GameCenterControllerDelegate`, and `GameCenterViewState`
- Typed `GameKit` error coverage via `ErrorCode`, `ERROR_DOMAIN`, and `GameKitFrameworkError::error_code`
- Exported GameKit constants for turn/exchange timeouts plus player/auth notification names and `PLAYER_ID_NO_LONGER_AVAILABLE`
- Invite-recipient response coverage via `InviteRecipientResponse` and `Matchmaker::find_*_with_recipient_responses`

### Changed
- Closed the 15 remaining audited top-level GameKit gaps and updated the coverage audit to 100%
- Refreshed the README, coverage notes, and smoke tests to reflect the completed surface
- Bumped the crate version to `0.2.2`

## [0.2.1] - 2026-05-16

### Added
- `GameActivity` bindings covering definition loading, activity snapshots, lifecycle control, matchmaking handoff, and deep-link metadata/image access on macOS 26
- `LocalPlayer::register_listener` with `LocalPlayerListener`/`LocalPlayerEvent` support for invite, turn-based, saved-game, and game-activity callbacks
- AppKit Game Center UI wrappers for `DialogController`, `Invite`, `MatchmakerViewController`, `TurnBasedMatchmakerViewController`, and their delegate/event types
- `14_game_activity` example plus smoke-test coverage for the new game-activity, listener, and AppKit UI surface

### Changed
- Updated the audit table to mark the Game Activity, local-player listener, invite, dialog-controller, and matchmaking UI gaps as verified
- Bumped the crate version to `0.2.1`

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
