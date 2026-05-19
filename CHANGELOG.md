# Changelog

## [0.8.6] - 2026-05-19

- Bump MSRV from 1.70 to 1.76 to match fleet baseline.

## [0.8.5] - 2026-05-19

### Changed
- Documented `GKGameSessionSharingViewController`, `GKGameSessionSharingViewControllerDelegate`, `GKPeerPickerController`, and `GKPeerPickerControllerDelegate` as deprecated legacy UI symbols intentionally left unwrapped because the GameKit headers make them tvOS/iOS-only and unavailable on macOS.
- Fixed the `GKLocalPlayerListener` game-activity witness so the Swift bridge builds again against the macOS 26 SDK.
- Bumped the crate version to `0.8.5`.

## [0.3.2] - 2026-05-19

### Fixed
- **Panic safety (UB fix)**: all 9 `extern "C"` trampolines that invoke user
  closures now wrap the call in `doom_fish_utils::panic_safe::catch_user_panic`.
  Previously a panic in a user-supplied handler would unwind through the
  Swift/C ABI boundary — undefined behaviour.  Affected trampolines:
  `auth_trampoline`, `match_data_trampoline`, `match_state_trampoline`,
  `match_failure_trampoline`, `game_center_trampoline`,
  `matchmaker_view_controller_trampoline`,
  `turn_based_matchmaker_view_controller_trampoline`,
  `invite_recipient_response_trampoline`,
  `local_player_listener_trampoline`.
- Added `// SAFETY:` justification comments to all 12 `unsafe impl Send`
  declarations (`MatchPtrSend`, `GameActivity`, `LocalPlayerListener`,
  `AuthObserver`, `Match`, `MatchDelegate`, `Invite`,
  `GameCenterControllerDelegate`, `MatchmakerViewControllerDelegate`,
  `MatchmakerViewController`, `TurnBasedMatchmakerViewControllerDelegate`,
  `TurnBasedMatchmakerViewController`).
- Widened `doom-fish-utils` version constraint from `"0.1"` to `">=0.1, <0.3"`
  to allow the next minor release without a breaking bump.

## [0.3.1] - 2026-05-19

### Changed
- Added `#if GAMEKIT_HAS_MACOS26_SDK` compile-time guard around the
  `wantsToPlay(_:completionHandler:)` method in `GKLocalPlayerListenerImpl`
  (`LocalPlayerListener.swift`). The method references `GKGameActivity` and
  `gkGameActivityPayload`, both of which are only available in the macOS 26
  SDK; the previous `@available(macOS 26.0, *)` runtime attribute alone was
  not sufficient to prevent compilation failures on older SDKs (macos-15 /
  Xcode 16).
- Fixed a stale README doctest: `AsyncLocalPlayer::new().authenticate()` →
  `AsyncLocalPlayer::authenticate()` (the method is a static, not an instance
  call), and corrected the accessed field to `player.player.display_name`.
- Fixed pre-existing `doc_markdown` Clippy warning (unbackticked `GameKit`)
  in `README.md` and `tests/async_api_tests.rs`.
- Fixed pre-existing `items_after_statements` Clippy warnings in
  `tests/async_api_tests.rs` by moving inner `struct`/`impl` blocks before
  the `let` bindings in each test function.

## [0.3.0] - 2026-05-18

### Added
- `async_api` module behind the `async` feature flag, wrapping 9 Apple GameKit completion-handler APIs as executor-agnostic Rust `Future` newtypes:
  - `AsyncLocalPlayer` — `authenticate` + `friends_authorization_status`
  - `AsyncMatchmaker` — `find_match` + `find_players`
  - `AsyncLeaderboard` — `load_leaderboards` + `load_entries`
  - `AsyncAchievement` — `load_achievements` + `report_achievement`
  - `AsyncSavedGame` — `fetch_all_saved_games`, `load_data`, `save_game`
- 11 Swift thunks in `Async.swift` using `@_cdecl` to fire C callbacks from Task closures
- 11 public `Future` newtypes (`AuthenticateFuture`, `FriendsAuthorizationFuture`, `FindMatchFuture`, `FindPlayersFuture`, `LoadLeaderboardsFuture`, `LoadEntriesFuture`, `LoadAchievementsFuture`, `ReportAchievementFuture`, `FetchAllSavedGamesFuture`, `LoadSavedGameDataFuture`, `SaveGameFuture`)
- 4 new async examples: `15_async_auth`, `16_async_leaderboard`, `17_async_achievement`, `18_async_saved_game`
- `doom-fish-utils` dependency providing the `AsyncCompletion` / `AsyncCompletionFuture` pattern
- `pollster = "0.3"` dev-dependency for blocking on futures in tests and examples

### Changed
- Bumped the crate version to `0.3.0`

## [0.2.3] - 2026-05-17

### Added
- Seven new integration tests under `tests/`, covering `LocalPlayer`, `Achievement`, `Leaderboard`, `Match`, `TurnBased`, `ChallengeDefinition`, and `GameActivity`

### Changed
- Bumped the crate version to `0.2.3`

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
