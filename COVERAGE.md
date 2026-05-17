# GameKit coverage audit

`gamekit-rs` v0.2.2 targets the GameKit APIs that are practical to expose from a safe Rust surface on macOS and now reaches 100% of the audited top-level macOS-available surface tracked in [`COVERAGE_AUDIT.md`](COVERAGE_AUDIT.md). The crate still mirrors the `screencapturekit-rs` bridge layout:

- one Swift bridge file per logical area
- `@_cdecl` C-callable entry points
- JSON payload exchange for structured data
- retained opaque handles for live `GKMatch` objects and UI controllers
- SDK gating for newer APIs such as `GKChallengeDefinition` and `GKGameActivity`

## Status by logical area

| Area | Apple surface audited | Rust surface in v0.2.2 | Status |
| --- | --- | --- | --- |
| Error | `GKErrorCode`, `GKErrorDomain` | `ErrorCode`, `ERROR_DOMAIN`, `GameKitFrameworkError::error_code` | Covered |
| LocalPlayer | `GKLocalPlayer` snapshot, auth handler, recent players, challengeable friends, identity verification signature, friends authorization, friends lookup, friend request presentation, auth-change notification constant | `LocalPlayer`, `AuthObserver`, `FriendsAuthorizationStatus`, `IdentityVerificationSignature`, `PLAYER_AUTHENTICATION_DID_CHANGE_NOTIFICATION_NAME` | Broad coverage of local-player data and exported constants |
| LocalPlayerListener | `GKLocalPlayerListener`, `GKInviteEventListener`, `GKTurnBasedEventListener`, `GKSavedGameListener`, `GKGameActivityListener` | `LocalPlayer::register_listener`, `LocalPlayerListener`, `LocalPlayerEvent` | Broad callback coverage for invite, turn-based, saved-game, and game-activity events |
| Player | `GKBasePlayer`, `GKPlayer`, `GKPhotoSize`, player notification constants | `BasePlayer`, `Player`, `PhotoSize`, `PLAYER_DID_CHANGE_NOTIFICATION_NAME`, `PLAYER_ID_NO_LONGER_AVAILABLE` | Covered for identity snapshots and exported constants; photo-loading members remain intentionally unwrapped |
| Leaderboard | `GKLeaderboard` load, recurring metadata, score submission | `Leaderboard`, `LeaderboardType`, `submit_score`, `submit_local_score`, `load_previous_occurrence` | Broad coverage of load + submit flows |
| LeaderboardSet | `GKLeaderboardSet` discovery, leaderboard loading, image loading | `LeaderboardSet`, `LeaderboardSet::load_*` | Covered |
| LeaderboardEntry | `GKLeaderboard.Entry` rank/score/date/player snapshots and load APIs | `LeaderboardEntry`, `LoadEntriesResult`, `load_entries`, `load_entries_for_players` | Covered |
| Achievement | `GKAchievement`, `GKAchievementDescription`, reset/report/load | `Achievement`, `AchievementDescription`, `load`, `load_descriptions`, `report`, `report_all`, `reset` | Core coverage; image loading / rarity metadata are not wrapped |
| Match | `GKMatch` players, connection state, data send, callbacks, rematch, host selection | `Match`, `MatchEvent`, `MatchDelegate`, `ConnectionState`, `SendDataMode` | Covered for the primary live-match surface |
| TurnBased | `GKTurnBasedMatch`, participants, exchanges, reminders, merged saves, quit/end flows, timeout constants | `TurnBasedMatch`, `TurnBasedParticipant`, `TurnBasedExchange`, `TurnBasedExchangeReply`, related enums/request types, `TURN_*`, `EXCHANGE_*` constants | Broad coverage |
| RealTime | `GKMatchmaker`, `GKInviteRecipientResponse`, `GKMatchedPlayers` | `Matchmaker`, `MatchRequest`, `MatchType`, `InviteRecipientResponse`, `MatchedPlayers` | Covered for the primary programmatic matchmaking surface; nearby-player browsing, queue activity, and invite cancellation remain member-level omissions |
| MatchmakingUI | `GKDialogController`, `GKInvite`, `GKMatchmakerViewController`, `GKTurnBasedMatchmakerViewController`, delegates, matchmaking mode, legacy `GKGameCenterControllerDelegate` | `DialogController`, `GameCenterControllerDelegate`, `GameCenterViewState`, `Invite`, `MatchmakerViewController`, `TurnBasedMatchmakerViewController`, `MatchmakingMode` | Covered for AppKit Game Center UI flows, including legacy dashboard dismissal callbacks |
| GameActivity | `GKGameActivity`, `GKGameActivityDefinition`, play style/state metadata, definition lookup, activity lifecycle, image loading | `GameActivity`, `GameActivityDefinition`, `GameActivitySnapshot`, `GameActivityPlayStyle`, `GameActivityState` | SDK/OS-gated: available only when compiled with a macOS 26 SDK and run on macOS 26+ |
| Notification | `GKNotificationBanner` | `NotificationBanner` | Covered via deprecated Apple API |
| AccessPoint | `GKAccessPoint` snapshot, activation, location, trigger, trigger(state:)` | `AccessPoint`, `AccessPointSnapshot`, `AccessPointLocation`, `AccessPointState` | Core coverage; newer state-trigger variants are not wrapped |
| ChallengeDefinition | `GKChallengeDefinition` load + active-challenge query | `ChallengeDefinition`, `ChallengeDurationOption` | SDK/OS-gated: available only when compiled with a macOS 26 SDK and run on macOS 26+; image loading not wrapped |
| Score | legacy `GKScore` reporting | `Score`, `Score::new_local`, `Score::for_player_game_id`, `Score::report_all` | Covered via deprecated Apple API |
| Save | `GKSavedGame` list, load data, save, delete, resolve conflicts | `SavedGame` | Broad coverage; saved-game listener callbacks are exposed through `LocalPlayerListener` |

## Deprecated Apple APIs intentionally exposed

These APIs are still available in GameKit and are often needed when interoperating with older Game Center integrations, so the crate exposes them and documents the deprecation instead of dropping them:

- `GKNotificationBanner` → `NotificationBanner`
- `GKScore` → `Score`
- the legacy `GKGameCenterControllerDelegate` dismissal flow via `DialogController::present_game_center_*`

## SDK and OS gating

Some GameKit surface is not universally available:

- `GKChallengeDefinition` and `GKGameActivity` are compiled only when the detected macOS SDK is 26 or newer.
- `MatchedPlayers` requires macOS 14.2 or newer at runtime.
- `DialogController::present_game_center_state(GameCenterViewState::LocalPlayerFriendsList, ...)` requires macOS 12.0 or newer.
- At runtime, the bridge returns `GameKitError::Unavailable` when a gated API is called on an older SDK or OS.

## Remaining known gaps

v0.2.2 closes every audited top-level macOS-available GameKit symbol in `COVERAGE_AUDIT.md`. The crate is still not a literal wrapper of every Apple member-level API. The main intentionally-unwrapped areas are:

- some image/photo convenience loaders outside `LeaderboardSet::load_image_data()` and the newer activity/challenge metadata flows
- newer nearby-player, queue/group-activity, and invite-cancellation helpers on `GKMatchmaker`
- a few access-point, challenge-definition, and leaderboard-set convenience helpers added in the latest SDKs

Those member-level omissions are isolated so future releases can extend the bridge without revisiting the overall crate structure.
