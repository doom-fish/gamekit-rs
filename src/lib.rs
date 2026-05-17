#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

pub mod access_point;
pub mod achievement;
pub mod base_player;
pub mod challenge_definition;
pub mod error;
mod ffi;
pub mod game_activity;

#[cfg(feature = "async")]
pub mod async_api;
pub mod gk_match;
pub mod leaderboard;
pub mod leaderboard_entry;
pub mod leaderboard_set;
pub mod local_player;
pub mod local_player_listener;
pub mod r#match;
pub mod matchmaker;
pub mod matchmaker_ui;
pub mod notification;
pub mod player;
mod private;
pub mod real_time;
pub mod save;
pub mod score;
pub mod turn_based;

pub use access_point::{
    AccessPoint, AccessPointFrame, AccessPointLocation, AccessPointSnapshot, AccessPointState,
};
pub use achievement::{Achievement, AchievementDescription};
pub use base_player::BasePlayer;
pub use challenge_definition::{ChallengeDefinition, ChallengeDurationOption};
pub use error::{ErrorCode, GameKitError, GameKitFrameworkError, ERROR_DOMAIN};
pub use game_activity::{
    GameActivity, GameActivityDefinition, GameActivityPlayStyle, GameActivitySnapshot,
    GameActivityState,
};
pub use leaderboard::{Leaderboard, LeaderboardType, PlayerScope, TimeScope};
pub use leaderboard_entry::{LeaderboardEntry, LoadEntriesResult};
pub use leaderboard_set::LeaderboardSet;
pub use local_player::{
    AuthEvent, AuthEventError, AuthObserver, FriendsAuthorizationStatus,
    IdentityVerificationSignature, LocalPlayer, PLAYER_AUTHENTICATION_DID_CHANGE_NOTIFICATION_NAME,
};
pub use local_player_listener::{LocalPlayerEvent, LocalPlayerListener};
pub use matchmaker_ui::{
    DialogController, GameCenterControllerDelegate, GameCenterViewState, Invite,
    MatchmakerViewController, MatchmakerViewControllerDelegate, MatchmakerViewControllerEvent,
    MatchmakingMode, TurnBasedMatchmakerViewController, TurnBasedMatchmakerViewControllerDelegate,
    TurnBasedMatchmakerViewControllerEvent,
};
pub use notification::NotificationBanner;
pub use player::{
    InviteRecipientResponse, PhotoSize, Player, PLAYER_DID_CHANGE_NOTIFICATION_NAME,
    PLAYER_ID_NO_LONGER_AVAILABLE,
};
pub use r#match::{ConnectionState, Match, MatchDelegate, MatchEvent, SendDataMode};
pub use real_time::{
    MatchProperties, MatchRequest, MatchType, MatchedPlayerProperties, MatchedPlayers, Matchmaker,
};
pub use save::SavedGame;
pub use score::Score;
pub use turn_based::{
    TurnBasedExchange, TurnBasedExchangeReply, TurnBasedExchangeStatus, TurnBasedMatch,
    TurnBasedMatchOutcome, TurnBasedMatchRequest, TurnBasedMatchStatus, TurnBasedParticipant,
    TurnBasedParticipantStatus, EXCHANGE_TIMEOUT_DEFAULT, EXCHANGE_TIMEOUT_NONE,
    TURN_TIMEOUT_DEFAULT, TURN_TIMEOUT_NONE,
};

/// Common imports.
pub mod prelude {
    pub use crate::access_point::{
        AccessPoint, AccessPointFrame, AccessPointLocation, AccessPointSnapshot, AccessPointState,
    };
    pub use crate::achievement::{Achievement, AchievementDescription};
    pub use crate::base_player::BasePlayer;
    pub use crate::challenge_definition::{ChallengeDefinition, ChallengeDurationOption};
    pub use crate::error::{ErrorCode, GameKitError, GameKitFrameworkError, ERROR_DOMAIN};
    pub use crate::game_activity::{
        GameActivity, GameActivityDefinition, GameActivityPlayStyle, GameActivitySnapshot,
        GameActivityState,
    };
    pub use crate::leaderboard::{Leaderboard, LeaderboardType, PlayerScope, TimeScope};
    pub use crate::leaderboard_entry::{LeaderboardEntry, LoadEntriesResult};
    pub use crate::leaderboard_set::LeaderboardSet;
    pub use crate::local_player::{
        AuthEvent, AuthEventError, AuthObserver, FriendsAuthorizationStatus,
        IdentityVerificationSignature, LocalPlayer,
        PLAYER_AUTHENTICATION_DID_CHANGE_NOTIFICATION_NAME,
    };
    pub use crate::local_player_listener::{LocalPlayerEvent, LocalPlayerListener};
    pub use crate::matchmaker_ui::{
        DialogController, GameCenterControllerDelegate, GameCenterViewState, Invite,
        MatchmakerViewController, MatchmakerViewControllerDelegate, MatchmakerViewControllerEvent,
        MatchmakingMode, TurnBasedMatchmakerViewController,
        TurnBasedMatchmakerViewControllerDelegate, TurnBasedMatchmakerViewControllerEvent,
    };
    pub use crate::notification::NotificationBanner;
    pub use crate::player::{
        InviteRecipientResponse, PhotoSize, Player, PLAYER_DID_CHANGE_NOTIFICATION_NAME,
        PLAYER_ID_NO_LONGER_AVAILABLE,
    };
    pub use crate::r#match::{ConnectionState, Match, MatchDelegate, MatchEvent, SendDataMode};
    pub use crate::real_time::{
        MatchProperties, MatchRequest, MatchType, MatchedPlayerProperties, MatchedPlayers,
        Matchmaker,
    };
    pub use crate::save::SavedGame;
    pub use crate::score::Score;
    pub use crate::turn_based::{
        TurnBasedExchange, TurnBasedExchangeReply, TurnBasedExchangeStatus, TurnBasedMatch,
        TurnBasedMatchOutcome, TurnBasedMatchRequest, TurnBasedMatchStatus, TurnBasedParticipant,
        TurnBasedParticipantStatus, EXCHANGE_TIMEOUT_DEFAULT, EXCHANGE_TIMEOUT_NONE,
        TURN_TIMEOUT_DEFAULT, TURN_TIMEOUT_NONE,
    };
}
