use core::ffi::c_char;

use serde::Deserialize;

use crate::{ffi, private, GameKitError};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinaryPayload {
    data_base64: String,
}

/// A duration option attached to a challenge definition.
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeDurationOption {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub week_of_year: Option<i32>,
    pub day: Option<i32>,
    pub hour: Option<i32>,
    pub minute: Option<i32>,
    pub second: Option<i32>,
}

/// Metadata for a developer-defined challenge.
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeDefinition {
    pub identifier: String,
    pub group_identifier: Option<String>,
    pub title: String,
    pub details: Option<String>,
    pub duration_options: Vec<ChallengeDurationOption>,
    pub is_repeatable: bool,
    pub leaderboard_id: Option<String>,
    pub release_state: String,
}

impl ChallengeDefinition {
    /// Loads every challenge definition for the current game.
    pub fn load_all() -> Result<Vec<Self>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_challenge_definitions_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "challenge definitions")
        }
    }

    /// Returns whether this challenge definition has active challenges.
    pub fn has_active_challenges(&self) -> Result<bool, GameKitError> {
        let identifier = private::cstring_from_str(&self.identifier, "challenge definition identifier")?;

        unsafe {
            let mut out_active = false;
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_challenge_definition_has_active(
                identifier.as_ptr(),
                &mut out_active,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(out_active)
        }
    }

    /// Loads the challenge definition image as TIFF bytes.
    pub fn load_image_data(&self) -> Result<Vec<u8>, GameKitError> {
        let identifier =
            private::cstring_from_str(&self.identifier, "challenge definition identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_challenge_definition_load_image_json(
                identifier.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payload: BinaryPayload =
                private::parse_json_ptr(out_json, "challenge definition image")?;
            private::decode_base64(&payload.data_base64, "challenge definition image")
        }
    }
}
