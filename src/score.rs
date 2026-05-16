use core::ffi::c_char;

use serde::{Deserialize, Serialize};

use crate::{ffi, private, GameKitError};

/// A score submission payload for Game Center leaderboards.
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub leaderboard_id: String,
    pub value: i64,
    pub context: u64,
    pub player_game_id: Option<String>,
}

impl Score {
    /// Creates a score for the local player.
    pub fn new_local(leaderboard_id: impl Into<String>, value: i64, context: u64) -> Self {
        Self {
            leaderboard_id: leaderboard_id.into(),
            value,
            context,
            player_game_id: None,
        }
    }

    /// Creates a score for a specific scoped player identifier.
    pub fn for_player_game_id(
        leaderboard_id: impl Into<String>,
        value: i64,
        context: u64,
        player_game_id: impl Into<String>,
    ) -> Self {
        Self {
            leaderboard_id: leaderboard_id.into(),
            value,
            context,
            player_game_id: Some(player_game_id.into()),
        }
    }

    /// Reports a slice of scores using the legacy `GKScore` API.
    pub fn report_all(scores: &[Self]) -> Result<(), GameKitError> {
        let json = private::json_cstring(scores, "scores")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_score_report_json(json.as_ptr(), &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }
}
