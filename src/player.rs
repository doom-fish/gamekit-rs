use serde::Deserialize;

/// Represents a player in Game Center.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    #[serde(rename = "gamePlayerID")]
    pub game_player_id: String,
    #[serde(rename = "teamPlayerID")]
    pub team_player_id: String,
    pub alias: String,
    pub display_name: String,
    #[serde(rename = "playerID")]
    pub player_id: Option<String>,
}
