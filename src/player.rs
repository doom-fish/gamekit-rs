use core::ffi::c_char;

use serde::{Deserialize, Serialize};

use crate::{ffi, private, GameKitError};

/// The exported player-change notification name.
pub const PLAYER_DID_CHANGE_NOTIFICATION_NAME: &str = "GKPlayerDidChangeNotificationName";

/// The exported sentinel string returned by deprecated player-ID APIs.
pub const PLAYER_ID_NO_LONGER_AVAILABLE: &str = "playerID is no longer available";

/// Available player photo sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoSize {
    Small,
    Normal,
}

/// Response for a programmatic invite recipient.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InviteRecipientResponse {
    Accepted,
    Declined,
    Failed,
    Incompatible,
    UnableToConnect,
    NoAnswer,
}

impl InviteRecipientResponse {
    pub(crate) const fn from_raw(value: i32) -> Self {
        match value {
            1 => Self::Declined,
            2 => Self::Failed,
            3 => Self::Incompatible,
            4 => Self::UnableToConnect,
            5 => Self::NoAnswer,
            _ => Self::Accepted,
        }
    }
}

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
