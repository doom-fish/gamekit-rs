use serde::{Deserialize, Serialize};

use crate::local_player::LocalPlayer;
use crate::player::Player;

/// Common Game Center base-player fields shared by `Player` and `LocalPlayer` snapshots.
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BasePlayer {
    pub display_name: Option<String>,
    pub player_id: Option<String>,
}

impl From<&Player> for BasePlayer {
    fn from(player: &Player) -> Self {
        Self {
            display_name: Some(player.display_name.clone()),
            player_id: player.player_id.clone(),
        }
    }
}

impl From<Player> for BasePlayer {
    fn from(player: Player) -> Self {
        Self::from(&player)
    }
}

impl From<&LocalPlayer> for BasePlayer {
    fn from(player: &LocalPlayer) -> Self {
        Self::from(&player.player)
    }
}

impl From<LocalPlayer> for BasePlayer {
    fn from(player: LocalPlayer) -> Self {
        Self::from(&player)
    }
}
