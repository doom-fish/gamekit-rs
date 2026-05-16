#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_panics_doc,
)]

pub mod achievement;
pub mod error;
pub mod gk_match;
pub mod leaderboard;
pub mod local_player;
pub mod matchmaker;
pub mod player;
mod ffi;
mod private;

pub use achievement::Achievement;
pub use error::GameKitError;
pub use gk_match::{ConnectionState, Match, MatchDelegate, MatchEvent, SendDataMode};
pub use leaderboard::{Leaderboard, LeaderboardEntry, LeaderboardType, LoadEntriesResult, PlayerScope, TimeScope};
pub use local_player::{AuthEvent, AuthObserver, LocalPlayer};
pub use matchmaker::{Matchmaker, MatchRequest};
pub use player::Player;

/// Common imports.
pub mod prelude {
    pub use crate::achievement::Achievement;
    pub use crate::error::GameKitError;
    pub use crate::gk_match::{ConnectionState, Match, MatchDelegate, MatchEvent, SendDataMode};
    pub use crate::leaderboard::{Leaderboard, LeaderboardEntry, LeaderboardType, LoadEntriesResult, PlayerScope, TimeScope};
    pub use crate::local_player::{AuthEvent, AuthObserver, LocalPlayer};
    pub use crate::matchmaker::{Matchmaker, MatchRequest};
    pub use crate::player::Player;
}
