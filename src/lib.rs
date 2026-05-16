#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
)]

pub mod access_point;
pub mod achievement;
pub mod challenge_definition;
pub mod error;
pub mod gk_match;
pub mod leaderboard;
pub mod leaderboard_entry;
pub mod local_player;
pub mod matchmaker;
pub mod notification;
pub mod player;
pub mod r#match;
pub mod real_time;
pub mod save;
pub mod score;
pub mod turn_based;
mod ffi;
mod private;

pub use access_point::{
    AccessPoint, AccessPointFrame, AccessPointLocation, AccessPointSnapshot, AccessPointState,
};
pub use achievement::{Achievement, AchievementDescription};
pub use challenge_definition::{ChallengeDefinition, ChallengeDurationOption};
pub use error::{GameKitError, GameKitFrameworkError};
pub use leaderboard::{Leaderboard, LeaderboardType, PlayerScope, TimeScope};
pub use leaderboard_entry::{LeaderboardEntry, LoadEntriesResult};
pub use local_player::{
    AuthEvent, AuthEventError, AuthObserver, FriendsAuthorizationStatus,
    IdentityVerificationSignature, LocalPlayer,
};
pub use notification::NotificationBanner;
pub use player::Player;
pub use r#match::{ConnectionState, Match, MatchDelegate, MatchEvent, SendDataMode};
pub use real_time::{MatchRequest, MatchType, Matchmaker};
pub use save::SavedGame;
pub use score::Score;
pub use turn_based::{
    TurnBasedExchange, TurnBasedExchangeReply, TurnBasedExchangeStatus, TurnBasedMatch,
    TurnBasedMatchOutcome, TurnBasedMatchRequest, TurnBasedMatchStatus, TurnBasedParticipant,
    TurnBasedParticipantStatus,
};

/// Common imports.
pub mod prelude {
    pub use crate::access_point::{
        AccessPoint, AccessPointFrame, AccessPointLocation, AccessPointSnapshot,
        AccessPointState,
    };
    pub use crate::achievement::{Achievement, AchievementDescription};
    pub use crate::challenge_definition::{ChallengeDefinition, ChallengeDurationOption};
    pub use crate::error::{GameKitError, GameKitFrameworkError};
    pub use crate::leaderboard::{Leaderboard, LeaderboardType, PlayerScope, TimeScope};
    pub use crate::leaderboard_entry::{LeaderboardEntry, LoadEntriesResult};
    pub use crate::local_player::{
        AuthEvent, AuthEventError, AuthObserver, FriendsAuthorizationStatus,
        IdentityVerificationSignature, LocalPlayer,
    };
    pub use crate::notification::NotificationBanner;
    pub use crate::player::Player;
    pub use crate::r#match::{ConnectionState, Match, MatchDelegate, MatchEvent, SendDataMode};
    pub use crate::real_time::{MatchRequest, MatchType, Matchmaker};
    pub use crate::save::SavedGame;
    pub use crate::score::Score;
    pub use crate::turn_based::{
        TurnBasedExchange, TurnBasedExchangeReply, TurnBasedExchangeStatus, TurnBasedMatch,
        TurnBasedMatchOutcome, TurnBasedMatchRequest, TurnBasedMatchStatus,
        TurnBasedParticipant, TurnBasedParticipantStatus,
    };
}
