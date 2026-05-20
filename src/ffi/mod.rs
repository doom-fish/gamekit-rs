#![allow(dead_code, missing_docs)]

mod access_point;
mod achievement;
#[cfg(feature = "async")]
mod async_ffi;
mod challenge_definition;
mod core;
mod game_activity;
mod leaderboard;
mod leaderboard_entry;
mod leaderboard_set;
mod local_player;
mod local_player_listener;
mod r#match;
mod matchmaker_ui;
mod notification;
mod player;
mod real_time;
mod save;
mod score;
mod turn_based;

pub use access_point::*;
pub use achievement::*;
#[cfg(feature = "async")]
pub use async_ffi::*;
pub use challenge_definition::*;
pub use core::*;
pub use game_activity::*;
pub use leaderboard::*;
pub use leaderboard_entry::*;
pub use leaderboard_set::*;
pub use local_player::*;
pub use local_player_listener::*;
pub use matchmaker_ui::*;
pub use notification::*;
pub use player::*;
pub use r#match::*;
pub use real_time::*;
pub use save::*;
pub use score::*;
pub use turn_based::*;

pub mod status {
    pub use super::core::status::*;
}
