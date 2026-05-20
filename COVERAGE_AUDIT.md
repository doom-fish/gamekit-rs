# gamekit-rs coverage audit (vs MacOSX26.5.sdk)

> Scope: top-level macOS-available classes, protocols, enums, and exported constants in `GameKit.framework`. Method-level members, category-only extensions, and deprecated typedef aliases (for example `GKInviteeResponse`) are excluded from the counts. Legacy non-macOS symbols surfaced by the umbrella header (`GKPeerPicker*`, `GKVoiceChatService`, `GKVoiceChatServiceError`, `GKSessionError`, `GKGameSessionSharingViewController*`) are also excluded from the counts; deprecated ones are documented below as out-of-scope **EXEMPT** items. If Apple left a symbol non-deprecated even though it mainly serves a deprecated type (for example `GKGameCenterControllerDelegate`), it remains a **GAP**.

SDK_PUBLIC_SYMBOLS: 101
VERIFIED: 63
GAPS: 0
EXEMPT: 38
COVERAGE_PCT: 100.0%

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| GKAccessPoint | class | `GKAccessPoint.h` | `AccessPoint` |
| GKAccessPointLocation | enum | `GKAccessPoint.h` | `AccessPointLocation` |
| GKAchievement | class | `GKAchievement.h` | `Achievement` |
| GKAchievementDescription | class | `GKAchievementDescription.h` | `AchievementDescription`, `Achievement::load_descriptions`, `AsyncAchievement::load_descriptions` |
| GKBasePlayer | class | `GKBasePlayer.h` | `BasePlayer` |
| GKChallengeDefinition | class | `GKChallengeDefinition.h` | `ChallengeDefinition`, `ChallengeDefinition::load_image_data`, `AsyncChallengeDefinition::load_image_data` |
| GKDialogController | class | `GKDialogController.h` | `DialogController` |
| GKErrorCode | enum | `GKError.h` | `ErrorCode`, `GameKitFrameworkError::error_code` |
| GKErrorDomain | constant | `GKError.h` | `ERROR_DOMAIN` |
| GKExchangeTimeoutDefault | constant | `GKTurnBasedMatch.h` | `EXCHANGE_TIMEOUT_DEFAULT` |
| GKExchangeTimeoutNone | constant | `GKTurnBasedMatch.h` | `EXCHANGE_TIMEOUT_NONE` |
| GKFriendsAuthorizationStatus | enum | `GKLocalPlayer.h` | `FriendsAuthorizationStatus` |
| GKGameActivity | class | `GKGameActivity.h` | `GameActivity`, `GameActivitySnapshot` |
| GKGameActivityDefinition | class | `GKGameActivityDefinition.h` | `GameActivityDefinition`, `GameActivityDefinition::load_*` |
| GKGameActivityListener | protocol | `GKGameActivityListener.h` | `LocalPlayerListener` via `LocalPlayerEvent::WantsToPlayGameActivity` |
| GKGameActivityPlayStyle | enum | `GKGameActivityPlayStyle.h` | `GameActivityPlayStyle` |
| GKGameActivityState | enum | `GKGameActivityState.h` | `GameActivityState` |
| GKGameCenterControllerDelegate | protocol | `GKGameCenterViewController.h` | `GameCenterControllerDelegate`, `DialogController::present_game_center_*` |
| GKInvite | class | `GKMatchmaker.h` | `Invite`, `MatchmakerViewController::from_invite` |
| GKInviteEventListener | protocol | `GKMatchmaker.h` | `LocalPlayerListener` via invite-related `LocalPlayerEvent` variants |
| GKInviteRecipientResponse | enum | `GKMatchmaker.h` | `InviteRecipientResponse`, `Matchmaker::find_*_with_recipient_responses` |
| GKLeaderboard | class | `GKLeaderboard.h` | `Leaderboard` |
| GKLeaderboardEntry | class | `GKLeaderboardEntry.h` | `LeaderboardEntry`, `Leaderboard::load_entries*` |
| GKLeaderboardPlayerScope | enum | `GKLeaderboard.h` | `PlayerScope` |
| GKLeaderboardScore | class | `GKLeaderboardScore.h` | `Score` (turn-based `end_match_in_turn`) |
| GKLeaderboardSet | class | `GKLeaderboardSet.h` | `LeaderboardSet`, `LeaderboardSet::load_*`, `AsyncLeaderboardSet::{load_leaderboards, load_image_data}` |
| GKLeaderboardTimeScope | enum | `GKLeaderboard.h` | `TimeScope` |
| GKLeaderboardType | enum | `GKLeaderboard.h` | `LeaderboardType` |
| GKLocalPlayer | class | `GKLocalPlayer.h` | `LocalPlayer`, `AuthObserver` |
| GKLocalPlayerListener | protocol | `GKLocalPlayer.h` | `LocalPlayer::register_listener`, `LocalPlayerListener` |
| GKMatch | class | `GKMatch.h` | `Match` |
| GKMatchDelegate | protocol | `GKMatch.h` | `MatchDelegate` via `Match::set_delegate` |
| GKMatchedPlayers | class | `GKMatchmaker.h` | `MatchedPlayers`, `Matchmaker::find_matched_players` |
| GKMatchRequest | class | `GKMatchmaker.h` | `MatchRequest`, `TurnBasedMatchRequest` |
| GKMatchSendDataMode | enum | `GKMatch.h` | `SendDataMode` |
| GKMatchType | enum | `GKMatchmaker.h` | `MatchType` |
| GKMatchmaker | class | `GKMatchmaker.h` | `Matchmaker` |
| GKMatchmakerViewController | class | `GKMatchmakerViewController.h` | `MatchmakerViewController` |
| GKMatchmakerViewControllerDelegate | protocol | `GKMatchmakerViewController.h` | `MatchmakerViewControllerDelegate` via `MatchmakerViewController::set_delegate` |
| GKMatchmakingMode | enum | `GKMatchmakerViewController.h` | `MatchmakingMode` |
| GKPhotoSize | enum | `GKPlayer.h` | `PhotoSize` |
| GKPlayer | class | `GKPlayer.h` | `Player` |
| GKPlayerAuthenticationDidChangeNotificationName | constant | `GKLocalPlayer.h` | `PLAYER_AUTHENTICATION_DID_CHANGE_NOTIFICATION_NAME` |
| GKPlayerConnectionState | enum | `GKMatch.h` | `ConnectionState` |
| GKPlayerDidChangeNotificationName | constant | `GKPlayer.h` | `PLAYER_DID_CHANGE_NOTIFICATION_NAME` |
| GKPlayerIDNoLongerAvailable | constant | `GKPlayer.h` | `PLAYER_ID_NO_LONGER_AVAILABLE` |
| GKReleaseState | enum | `GKReleaseState.h` | `ChallengeDefinition.release_state` (stringified), `GameActivityDefinition.release_state` |
| GKSavedGame | class | `GKSavedGame.h` | `SavedGame` |
| GKSavedGameListener | protocol | `GKSavedGameListener.h` | `LocalPlayerListener` via saved-game `LocalPlayerEvent` variants |
| GKTurnBasedEventListener | protocol | `GKTurnBasedMatch.h` | `LocalPlayerListener` via turn/exchange `LocalPlayerEvent` variants |
| GKTurnBasedExchange | class | `GKTurnBasedMatch.h` | `TurnBasedExchange` |
| GKTurnBasedExchangeReply | class | `GKTurnBasedMatch.h` | `TurnBasedExchangeReply` |
| GKTurnBasedExchangeStatus | enum | `GKTurnBasedMatch.h` | `TurnBasedExchangeStatus` |
| GKTurnBasedMatch | class | `GKTurnBasedMatch.h` | `TurnBasedMatch` |
| GKTurnBasedMatchOutcome | enum | `GKTurnBasedMatch.h` | `TurnBasedMatchOutcome` |
| GKTurnBasedMatchStatus | enum | `GKTurnBasedMatch.h` | `TurnBasedMatchStatus` |
| GKTurnBasedMatchmakerViewController | class | `GKTurnBasedMatchmakerViewController.h` | `TurnBasedMatchmakerViewController` |
| GKTurnBasedMatchmakerViewControllerDelegate | protocol | `GKTurnBasedMatchmakerViewController.h` | `TurnBasedMatchmakerViewControllerDelegate` via `TurnBasedMatchmakerViewController::set_delegate` |
| GKTurnBasedParticipant | class | `GKTurnBasedMatch.h` | `TurnBasedParticipant` |
| GKTurnBasedParticipantStatus | enum | `GKTurnBasedMatch.h` | `TurnBasedParticipantStatus` |
| GKTurnTimeoutDefault | constant | `GKTurnBasedMatch.h` | `TURN_TIMEOUT_DEFAULT` |
| GKTurnTimeoutNone | constant | `GKTurnBasedMatch.h` | `TURN_TIMEOUT_NONE` |
| GKViewController | protocol | `GKDialogController.h` | `DialogController::present_*` with Game Center view-controller wrappers |

## 🔴 GAPS
None.

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

### Out-of-scope deprecated symbols (excluded from counts)
These legacy UI symbols are visible from the umbrella headers but are not macOS-available, so they do not change the macOS coverage totals above.

| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| GKGameSessionSharingViewController | class | `GKGameSessionSharingViewController.h` | Deprecated `GKGameSession` sharing UI; the header only exposes it on tvOS, so it is intentionally not wrapped for macOS. | `API_DEPRECATED(... tvos(10.0,12.0))` + `#if TARGET_OS_TV` |
| GKGameSessionSharingViewControllerDelegate | protocol | `GKGameSessionSharingViewController.h` | Delegate for the deprecated tvOS-only `GKGameSession` sharing UI; intentionally not wrapped for macOS. | `API_DEPRECATED(... tvos(10.0,12.0))` + `#if TARGET_OS_TV` |
| GKPeerPickerController | class | `GKPeerPickerController.h` | Deprecated legacy `GKSession` peer-picker UI; the header is iOS-only and unavailable on macOS, so it is intentionally not wrapped. | `API_DEPRECATED(... ios(3.0,7.0))` + `#if !TARGET_OS_OSX && !TARGET_OS_TV && !TARGET_OS_WATCH` |
| GKPeerPickerControllerDelegate | protocol | `GKPeerPickerController.h` | Delegate for the deprecated legacy `GKSession` peer-picker UI; iOS-only and intentionally not wrapped for macOS. | `API_DEPRECATED(... ios(3.0,7.0))` + `#if !TARGET_OS_OSX && !TARGET_OS_TV && !TARGET_OS_WATCH` |
