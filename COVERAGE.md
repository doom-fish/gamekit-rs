# GameKit coverage audit

`gamekit-rs` v0.2.1 targets the GameKit APIs that are practical to expose from a safe Rust surface on macOS and mirrors the `screencapturekit-rs` bridge layout:

- one Swift bridge file per logical area
- `@_cdecl` C-callable entry points
- JSON payload exchange for structured data
- retained opaque handles for live `GKMatch` objects
- SDK gating for newer APIs such as `GKChallengeDefinition` and `GKGameActivity`

## Status by logical area

| Area | Apple surface audited | Rust surface in v0.2.1 | Status |
| --- | --- | --- | --- |
| LocalPlayer | `GKLocalPlayer` snapshot, auth handler, recent players, challengeable friends, identity verification signature, friends authorization, friends lookup, friend request presentation | `LocalPlayer`, `AuthObserver`, `FriendsAuthorizationStatus`, `IdentityVerificationSignature` | Broad coverage of local-player data and auth flows |
| LocalPlayerListener | `GKLocalPlayerListener`, `GKInviteEventListener`, `GKTurnBasedEventListener`, `GKSavedGameListener`, `GKGameActivityListener` | `LocalPlayer::register_listener`, `LocalPlayerListener`, `LocalPlayerEvent` | Broad callback coverage for invite, turn-based, saved-game, and game-activity events |
| Player | `GKPlayer` identity fields, anonymous guest creation | `Player`, `Player::anonymous_guest` | Core identity coverage; photo/image loading not wrapped |
| Leaderboard | `GKLeaderboard` load, recurring metadata, score submission | `Leaderboard`, `LeaderboardType`, `submit_score`, `submit_local_score`, `load_previous_occurrence` | Broad coverage of load + submit flows |
| LeaderboardEntry | `GKLeaderboard.Entry` rank/score/date/player snapshots and load APIs | `LeaderboardEntry`, `LoadEntriesResult`, `load_entries`, `load_entries_for_players` | Covered |
| Achievement | `GKAchievement`, `GKAchievementDescription`, reset/report/load | `Achievement`, `AchievementDescription`, `load`, `load_descriptions`, `report`, `report_all`, `reset` | Core coverage; image loading / rarity metadata are not wrapped |
| Match | `GKMatch` players, connection state, data send, callbacks, rematch, host selection | `Match`, `MatchEvent`, `MatchDelegate`, `ConnectionState`, `SendDataMode` | Covered for the primary live-match surface |
| TurnBased | `GKTurnBasedMatch`, participants, exchanges, reminders, merged saves, quit/end flows | `TurnBasedMatch`, `TurnBasedParticipant`, `TurnBasedExchange`, `TurnBasedExchangeReply` and related enums/request types | Broad coverage |
| RealTime | `GKMatchmaker` find/cancel/finish/add/query/max-player helpers | `Matchmaker`, `MatchRequest`, `MatchType` | Broad coverage; nearby-player browsing, queue/group activity, invite cancellation, and matched-player APIs are still missing |
| MatchmakingUI | `GKDialogController`, `GKInvite`, `GKMatchmakerViewController`, `GKTurnBasedMatchmakerViewController`, delegates, matchmaking mode | `DialogController`, `Invite`, `MatchmakerViewController`, `TurnBasedMatchmakerViewController`, `MatchmakingMode` | Covered for the primary AppKit Game Center matchmaking UI flows |
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

## SDK and OS gating

Some GameKit surface is not universally available:

- `GKChallengeDefinition` and `GKGameActivity` are compiled only when the detected macOS SDK is 26 or newer.
- At runtime, the bridge returns `GameKitError::Unavailable` when a gated API is called on an older SDK or OS.

## Remaining known gaps

The crate is substantially broader in v0.2.1, but it is not a literal 100% wrapper of every Apple symbol. The main intentionally-unwrapped areas are:

- some image/photo loading APIs outside the new `GameActivityDefinition::load_image_data()` coverage
- newer matchmaking queue, nearby-player, and matched-player helpers
- typed error/constants wrappers such as `GKErrorCode`, timeout constants, and some notification constants
- a few access-point, challenge-definition, and leaderboard-set convenience helpers added in the latest SDKs

Those gaps are isolated and documented so future releases can extend the bridge without revisiting the overall crate structure.
