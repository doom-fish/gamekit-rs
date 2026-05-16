use core::ffi::{c_char, c_void};

use serde::Serialize;

use crate::{ffi, gk_match::Match, private, GameKitError};

/// Request parameters for finding a match.
#[derive(Debug, Clone)]
pub struct MatchRequest {
    pub min_players: u32,
    pub max_players: u32,
    pub player_group: u32,
    pub player_attributes: u32,
}

impl Default for MatchRequest {
    fn default() -> Self {
        Self {
            min_players: 2,
            max_players: 4,
            player_group: 0,
            player_attributes: 0,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchRequestPayload {
    min_players: u32,
    max_players: u32,
    player_group: u32,
    player_attributes: u32,
}

/// Provides access to the Game Center matchmaking service.
pub struct Matchmaker;

impl Matchmaker {
    /// Returns the shared matchmaker instance.
    #[must_use]
    pub const fn shared() -> Self {
        Self
    }

    /// Finds a match for the given request.
    pub fn find_match(&self, request: &MatchRequest) -> Result<Match, GameKitError> {
        let payload = MatchRequestPayload {
            min_players: request.min_players,
            max_players: request.max_players,
            player_group: request.player_group,
            player_attributes: request.player_attributes,
        };

        let json = private::json_cstring(&payload, "match request")?;

        unsafe {
            let mut out_match_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_find_match_json(
                json.as_ptr(),
                &mut out_match_ptr,
                &mut out_error,
            );

            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            Ok(Match::from_raw(out_match_ptr))
        }
    }

    /// Cancels the current matchmaking request.
    pub fn cancel(&self) {
        unsafe {
            ffi::gk_matchmaker_cancel();
        }
    }
}
