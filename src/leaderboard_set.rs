use core::ffi::c_char;

use serde::Deserialize;

use crate::{ffi, private, GameKitError, Leaderboard};

/// Metadata for a Game Center leaderboard set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderboardSet {
    pub title: String,
    pub group_identifier: Option<String>,
    pub identifier: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LeaderboardSetPayload {
    title: String,
    group_identifier: Option<String>,
    identifier: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinaryPayload {
    data_base64: String,
}

impl LeaderboardSet {
    /// Loads every leaderboard set available to the current game.
    pub fn load_all() -> Result<Vec<Self>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_leaderboard_sets_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payloads: Vec<LeaderboardSetPayload> =
                private::parse_json_ptr(out_json, "leaderboard sets")?;
            Ok(payloads.into_iter().map(Self::from_payload).collect())
        }
    }

    /// Loads the leaderboards contained in this set.
    pub fn load_leaderboards(&self) -> Result<Vec<Leaderboard>, GameKitError> {
        let identifier = self.identifier.as_deref().ok_or_else(|| {
            GameKitError::NotFound("leaderboard set identifier is missing".to_owned())
        })?;
        let identifier = private::cstring_from_str(identifier, "leaderboard set identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_leaderboard_set_load_leaderboards_json(
                identifier.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payloads: Vec<crate::leaderboard::LeaderboardPayload> =
                private::parse_json_ptr(out_json, "leaderboards in set")?;
            Ok(payloads
                .into_iter()
                .map(Leaderboard::from_payload)
                .collect())
        }
    }

    /// Loads the set image data as TIFF bytes.
    pub fn load_image_data(&self) -> Result<Vec<u8>, GameKitError> {
        let identifier = self.identifier.as_deref().ok_or_else(|| {
            GameKitError::NotFound("leaderboard set identifier is missing".to_owned())
        })?;
        let identifier = private::cstring_from_str(identifier, "leaderboard set identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_leaderboard_set_load_image_json(
                identifier.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payload: BinaryPayload =
                private::parse_json_ptr(out_json, "leaderboard set image")?;
            private::decode_base64(&payload.data_base64, "leaderboard set image")
        }
    }

    fn from_payload(payload: LeaderboardSetPayload) -> Self {
        Self {
            title: payload.title,
            group_identifier: payload.group_identifier,
            identifier: payload.identifier,
        }
    }
}
