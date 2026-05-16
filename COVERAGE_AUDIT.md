# gamekit-rs coverage audit (vs MacOSX26.2.sdk)

> Scope: top-level macOS-available classes, protocols, enums, and exported constants in `GameKit.framework`. Method-level members, category-only extensions, deprecated typedef aliases (for example `GKInviteeResponse`), and macOS-unavailable symbols (`GKPeerPicker*`, `GKVoiceChatService`, `GKVoiceChatServiceError`, `GKSessionError`, `GKGameSessionSharingViewController*`) are excluded from the counts. Deprecated symbols remain listed as **EXEMPT**. If Apple left a symbol non-deprecated even though it mainly serves a deprecated type (for example `GKGameCenterControllerDelegate`), it remains a **GAP**.

SDK_PUBLIC_SYMBOLS: 101
VERIFIED: 31
GAPS: 32
EXEMPT: 38
COVERAGE_PCT: 49.2%

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| GKAccessPoint | class | `GKAccessPoint.h` | `AccessPoint` |
| GKAccessPointLocation | enum | `GKAccessPoint.h` | `AccessPointLocation` |
| GKAchievement | class | `GKAchievement.h` | `Achievement` |
| GKAchievementDescription | class | `GKAchievementDescription.h` | `AchievementDescription` |
| GKChallengeDefinition | class | `GKChallengeDefinition.h` | `ChallengeDefinition` |
| GKFriendsAuthorizationStatus | enum | `GKLocalPlayer.h` | `FriendsAuthorizationStatus` |
| GKLeaderboard | class | `GKLeaderboard.h` | `Leaderboard` |
| GKLeaderboardEntry | class | `GKLeaderboardEntry.h` | `LeaderboardEntry`, `Leaderboard::load_entries*` |
| GKLeaderboardPlayerScope | enum | `GKLeaderboard.h` | `PlayerScope` |
| GKLeaderboardScore | class | `GKLeaderboardScore.h` | `Score` (turn-based `end_match_in_turn`) |
| GKLeaderboardTimeScope | enum | `GKLeaderboard.h` | `TimeScope` |
| GKLeaderboardType | enum | `GKLeaderboard.h` | `LeaderboardType` |
| GKLocalPlayer | class | `GKLocalPlayer.h` | `LocalPlayer`, `AuthObserver` |
| GKMatch | class | `GKMatch.h` | `Match` |
| GKMatchDelegate | protocol | `GKMatch.h` | `MatchDelegate` via `Match::set_delegate` |
| GKMatchRequest | class | `GKMatchmaker.h` | `MatchRequest`, `TurnBasedMatchRequest` |
| GKMatchSendDataMode | enum | `GKMatch.h` | `SendDataMode` |
| GKMatchType | enum | `GKMatchmaker.h` | `MatchType` |
| GKMatchmaker | class | `GKMatchmaker.h` | `Matchmaker` |
| GKPlayer | class | `GKPlayer.h` | `Player` |
| GKPlayerConnectionState | enum | `GKMatch.h` | `ConnectionState` |
| GKReleaseState | enum | `GKReleaseState.h` | `ChallengeDefinition.release_state` (stringified) |
| GKSavedGame | class | `GKSavedGame.h` | `SavedGame` |
| GKTurnBasedExchange | class | `GKTurnBasedMatch.h` | `TurnBasedExchange` |
| GKTurnBasedExchangeReply | class | `GKTurnBasedMatch.h` | `TurnBasedExchangeReply` |
| GKTurnBasedExchangeStatus | enum | `GKTurnBasedMatch.h` | `TurnBasedExchangeStatus` |
| GKTurnBasedMatch | class | `GKTurnBasedMatch.h` | `TurnBasedMatch` |
| GKTurnBasedMatchOutcome | enum | `GKTurnBasedMatch.h` | `TurnBasedMatchOutcome` |
| GKTurnBasedMatchStatus | enum | `GKTurnBasedMatch.h` | `TurnBasedMatchStatus` |
| GKTurnBasedParticipant | class | `GKTurnBasedMatch.h` | `TurnBasedParticipant` |
| GKTurnBasedParticipantStatus | enum | `GKTurnBasedMatch.h` | `TurnBasedParticipantStatus` |

## 🔴 GAPS
| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |
| GKBasePlayer | class | `GKBasePlayer.h` | No dedicated base-player abstraction; `Player` and `LocalPlayer` flatten the useful fields. |
| GKDialogController | class | `GKDialogController.h` | No AppKit dialog-controller helper is exposed. |
| GKErrorCode | enum | `GKError.h` | Errors surface as raw codes in `GameKitFrameworkError`, not as the typed SDK enum. |
| GKErrorDomain | constant | `GKError.h` | The crate exposes framework error domains as strings, not the exported SDK constant. |
| GKExchangeTimeoutDefault | constant | `GKTurnBasedMatch.h` | Exchange timeout convenience constants are not exposed. |
| GKExchangeTimeoutNone | constant | `GKTurnBasedMatch.h` | Exchange timeout convenience constants are not exposed. |
| GKGameActivity | class | `GKGameActivity.h` | The new macOS 26 game-activity instance APIs are entirely unwrapped. |
| GKGameActivityDefinition | class | `GKGameActivityDefinition.h` | No binding for activity definitions, deep-link metadata, or image loading. |
| GKGameActivityListener | protocol | `GKGameActivityListener.h` | Activity listener callbacks are not bridged. |
| GKGameActivityPlayStyle | enum | `GKGameActivityPlayStyle.h` | No Rust enum wrapper for activity play styles. |
| GKGameActivityState | enum | `GKGameActivityState.h` | No Rust enum wrapper for activity state. |
| GKGameCenterControllerDelegate | protocol | `GKGameCenterViewController.h` | Legacy Game Center UI delegate is not bridged; only `AccessPoint` flows are exposed. |
| GKInvite | class | `GKMatchmaker.h` | Accepted invites are not surfaced as Rust values. |
| GKInviteEventListener | protocol | `GKMatchmaker.h` | Invite listener callbacks are not wrapped. |
| GKInviteRecipientResponse | enum | `GKMatchmaker.h` | `recipientResponseHandler` is not exposed on `MatchRequest`. |
| GKLeaderboardSet | class | `GKLeaderboardSet.h` | Leaderboard-set discovery and image-loading APIs are missing. |
| GKLocalPlayerListener | protocol | `GKLocalPlayer.h` | `registerListener`/`unregisterListener` are intentionally omitted from the public API. |
| GKMatchedPlayers | class | `GKMatchmaker.h` | Rule-based matchmaking results are not wrapped. |
| GKMatchmakerViewController | class | `GKMatchmakerViewController.h` | No UI matchmaking controller wrapper exists. |
| GKMatchmakerViewControllerDelegate | protocol | `GKMatchmakerViewController.h` | No delegate bridge exists for the matchmaking UI controller. |
| GKMatchmakingMode | enum | `GKMatchmakerViewController.h` | Matchmaking UI mode selection is not wrapped. |
| GKPhotoSize | enum | `GKPlayer.h` | Player photo-loading APIs are intentionally unwrapped. |
| GKPlayerAuthenticationDidChangeNotificationName | constant | `GKLocalPlayer.h` | Authentication observation uses `authenticateHandler`, not the notification symbol. |
| GKPlayerDidChangeNotificationName | constant | `GKPlayer.h` | Player-change notifications are not exposed. |
| GKPlayerIDNoLongerAvailable | constant | `GKPlayer.h` | The sentinel constant for deprecated player IDs is not exposed. |
| GKSavedGameListener | protocol | `GKSavedGameListener.h` | Saved-game listener callbacks are not wrapped. |
| GKTurnBasedEventListener | protocol | `GKTurnBasedMatch.h` | Turn-based listener callbacks are not bridged. |
| GKTurnBasedMatchmakerViewController | class | `GKTurnBasedMatchmakerViewController.h` | No turn-based matchmaking UI wrapper exists. |
| GKTurnBasedMatchmakerViewControllerDelegate | protocol | `GKTurnBasedMatchmakerViewController.h` | No delegate bridge exists for turn-based matchmaking UI. |
| GKTurnTimeoutDefault | constant | `GKTurnBasedMatch.h` | Turn timeout convenience constants are not exposed. |
| GKTurnTimeoutNone | constant | `GKTurnBasedMatch.h` | Turn timeout convenience constants are not exposed. |
| GKViewController | protocol | `GKDialogController.h` | No AppKit `GKViewController` protocol bridge is exposed. |

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| GKAchievementChallenge | class | `GKChallenge.h` | Replaced by developer-defined challenges. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,26.0))` |
| GKAchievementViewController | class | `GKAchievementViewController.h` | Replaced by `GKGameCenterViewController`. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,10.10))` |
| GKAchievementViewControllerDelegate | protocol | `GKAchievementViewController.h` | Delegate for deprecated achievement UI. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,10.10))` |
| GKChallenge | class | `GKChallenge.h` | Replaced by developer-defined challenges. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,26.0))` |
| GKChallengeEventHandler | class | `GKChallengeEventHandler.h` | Replaced by `GKLocalPlayer` listener registration. | `API_DEPRECATED(... macos(10.8,10.10))` |
| GKChallengeEventHandlerDelegate | protocol | `GKChallengeEventHandler.h` | Delegate for deprecated challenge event handling. | `API_DEPRECATED(... macos(10.8,10.10))` |
| GKChallengeListener | protocol | `GKEventListener.h` | Replaced by developer-defined challenges. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.10,26.0))` |
| GKChallengeState | enum | `GKChallenge.h` | Replaced by developer-defined challenges. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,26.0))` |
| GKChallengesViewController | class | `GKChallengesViewController.h` | No longer supported legacy challenges UI. | `API_DEPRECATED(... macos(10.8,10.10))` |
| GKChallengesViewControllerDelegate | protocol | `GKChallengesViewController.h` | Delegate for deprecated challenges UI. | `API_DEPRECATED(... macos(10.8,15.4))` |
| GKCloudPlayer | class | `GKCloudPlayer.h` | Part of the deprecated `GKGameSession` surface. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.12,10.14))` |
| GKConnectionState | enum | `GKGameSession.h` | Part of the deprecated `GKGameSession` surface. | `API_DEPRECATED(... macos(10.12,10.14))` |
| GKFriendRequestComposeViewController | class | `GKFriendRequestComposeViewController.h` | No longer supported legacy friend-request UI. | `API_DEPRECATED(... macos(10.8,10.12))` |
| GKFriendRequestComposeViewControllerDelegate | protocol | `GKFriendRequestComposeViewController.h` | Delegate for deprecated friend-request UI. | `API_DEPRECATED(... macos(10.8,10.12))` |
| GKGameCenterViewController | class | `GKGameCenterViewController.h` | Replaced by `GKAccessPoint`. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.9,26.0))` |
| GKGameCenterViewControllerState | enum | `GKGameCenterViewController.h` | Part of the deprecated `GKGameCenterViewController` surface. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,26.0))` |
| GKGameSession | class | `GKGameSession.h` | Replaced by realtime and turn-based matchmaking APIs. | `API_DEPRECATED(... macos(10.12,10.14))` |
| GKGameSessionErrorCode | enum | `GKGameSessionError.h` | Error enum for deprecated `GKGameSession`. | `API_DEPRECATED(... macos(10.12,10.14))` |
| GKGameSessionErrorDomain | constant | `GKGameSessionError.h` | Error domain for deprecated `GKGameSession`. | `API_DEPRECATED(... macos(10.12,10.14))` |
| GKGameSessionEventListener | protocol | `GKGameSessionEventListener.h` | Replaced by `GKLocalPlayerListener`. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.12,10.14))` |
| GKLeaderboardViewController | class | `GKLeaderboardViewController.h` | Replaced by `GKGameCenterViewController`. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,10.10))` |
| GKLeaderboardViewControllerDelegate | protocol | `GKLeaderboardViewController.h` | Delegate for deprecated leaderboard UI. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,10.10))` |
| GKNotificationBanner | class | `GKNotificationBanner.h` | Replaced by UserNotifications or custom UI. | `API_DEPRECATED(... macos(10.8,14.0))` |
| GKPeerConnectionState | enum | `GKPublicConstants.h` | Legacy `GKSession` networking enum. | `API_DEPRECATED(... macos(10.8,10.10))` |
| GKScore | class | `GKScore.h` | Replaced by `GKLeaderboard.Entry` and `GKLeaderboardScore`. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,11.0))` |
| GKScoreChallenge | class | `GKChallenge.h` | Replaced by developer-defined challenges. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,26.0))` |
| GKSendDataMode | enum | `GKPublicConstants.h` | Legacy `GKSession` networking enum. | `API_DEPRECATED(... macos(10.8,10.10))` |
| GKSession | class | `GKSession.h` | Replaced by `GKMatch`. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,10.10))` |
| GKSessionDelegate | protocol | `GKPublicProtocols.h` | Replaced by `GKMatchDelegate`. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,10.10))` |
| GKSessionErrorDomain | constant | `GKSessionError.h` | Legacy `GKSession` error domain. | `API_DEPRECATED(... macos(10.10,15.4))` |
| GKSessionMode | enum | `GKPublicConstants.h` | Legacy `GKSession` networking enum. | `API_DEPRECATED(... macos(10.8,10.10))` |
| GKTransportType | enum | `GKGameSession.h` | Part of the deprecated `GKGameSession` surface. | `API_DEPRECATED(... macos(10.12,10.14))` |
| GKTurnBasedEventHandler | class | `GKTurnBasedMatch.h` | Replaced by `GKLocalPlayer` listener registration. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,10.10))` |
| GKTurnBasedEventHandlerDelegate | protocol | `GKTurnBasedMatch.h` | Delegate for deprecated turn-based event handling. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.8,10.10))` |
| GKVoiceChat | class | `GKVoiceChat.h` | No longer supported. | `API_DEPRECATED(... macos(10.8,15.0))` |
| GKVoiceChatClient | protocol | `GKPublicProtocols.h` | Legacy voice-chat client protocol. | `API_DEPRECATED_WITH_REPLACEMENT(... macos(10.6,10.8))` |
| GKVoiceChatPlayerState | enum | `GKVoiceChat.h` | No longer supported. | `API_DEPRECATED(... macos(10.8,15.0))` |
| GKVoiceChatServiceErrorDomain | constant | `GKPublicConstants.h` | Legacy VoiceChatService error domain. | `API_DEPRECATED(... macos(10.10,15.4))` |
