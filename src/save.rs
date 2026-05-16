use core::ffi::c_char;

use serde::{Deserialize, Serialize};

use crate::{ffi, private, GameKitError};

/// A saved-game snapshot.
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SavedGame {
    pub name: Option<String>,
    pub device_name: Option<String>,
    pub modification_date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedGameDataPayload {
    data_base64: String,
}

impl SavedGame {
    /// Loads all saved games for the local player.
    pub fn fetch_all() -> Result<Vec<Self>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_saved_games_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "saved games")
        }
    }

    /// Loads the data for this saved game snapshot.
    pub fn load_data(&self) -> Result<Vec<u8>, GameKitError> {
        let saved_game_json = private::json_cstring(self, "saved game")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_saved_game_load_data_json(
                saved_game_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payload: SavedGameDataPayload =
                private::parse_json_ptr(out_json, "saved game data")?;
            private::decode_base64(&payload.data_base64, "saved game")
        }
    }

    /// Saves a named saved-game payload.
    pub fn save(name: &str, data: &[u8]) -> Result<Self, GameKitError> {
        let name = private::cstring_from_str(name, "saved game name")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_saved_game_save_json(
                name.as_ptr(),
                data.as_ptr(),
                data.len(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "saved game")
        }
    }

    /// Deletes every saved game with the supplied name.
    pub fn delete(name: &str) -> Result<(), GameKitError> {
        let name = private::cstring_from_str(name, "saved game name")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_saved_game_delete(name.as_ptr(), &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Resolves a saved-game conflict set using the supplied merged data.
    pub fn resolve_conflicts(conflicting_games: &[Self], data: &[u8]) -> Result<Vec<Self>, GameKitError> {
        let conflicting_games_json = private::json_cstring(conflicting_games, "conflicting saved games")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_saved_game_resolve_conflicts_json(
                conflicting_games_json.as_ptr(),
                data.as_ptr(),
                data.len(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "resolved saved games")
        }
    }
}
