use core::ffi::c_char;

use serde::{Deserialize, Serialize};

use crate::{ffi, private, GameKitError};

/// Represents a player in Game Center.
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    #[serde(default)]
    pub guest_identifier: Option<String>,
    #[serde(default)]
    pub is_invitable: bool,
    #[serde(default)]
    pub scoped_ids_are_persistent: bool,
}

impl Player {
    /// Creates an anonymous guest player snapshot for the supplied guest identifier.
    pub fn anonymous_guest(identifier: &str) -> Result<Self, GameKitError> {
        let identifier = private::cstring_from_str(identifier, "guest identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_player_anonymous_guest_json(
                identifier.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "anonymous guest player")
        }
    }
}
