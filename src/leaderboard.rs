use core::ffi::c_char;

use serde::Deserialize;

use crate::{ffi, private, GameKitError};

/// Type of leaderboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderboardType {
    Classic,
    Recurring,
}

/// Scope for filtering leaderboard entries by player relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerScope {
    Global,
    FriendsOnly,
}

/// Time scope for filtering leaderboard entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeScope {
    Today,
    Week,
    AllTime,
}

/// Represents a Game Center leaderboard.
#[derive(Debug, Clone, PartialEq)]
pub struct Leaderboard {
    pub base_leaderboard_id: String,
    pub title: Option<String>,
    pub group_identifier: Option<String>,
    pub leaderboard_type: LeaderboardType,
    pub start_date: Option<String>,
    pub next_start_date: Option<String>,
    pub duration_seconds: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LeaderboardPayload {
    base_leaderboard_id: String,
    title: Option<String>,
    group_identifier: Option<String>,
    leaderboard_type: String,
    start_date: Option<String>,
    next_start_date: Option<String>,
    duration_seconds: Option<f64>,
}

impl Leaderboard {
    /// Loads leaderboards by their identifiers.
    pub fn load(ids: &[&str]) -> Result<Vec<Self>, GameKitError> {
        let ids_json = if ids.is_empty() {
            None
        } else {
            Some(private::json_cstring(ids, "leaderboard identifiers")?)
        };

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_leaderboard_load_json(
                ids_json.as_ref().map_or(std::ptr::null(), |json| json.as_ptr()),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payloads: Vec<LeaderboardPayload> =
                private::parse_json_ptr(out_json, "leaderboards")?;
            Ok(payloads.into_iter().map(Self::from_payload).collect())
        }
    }

    /// Loads every leaderboard available to the current game.
    pub fn load_all() -> Result<Vec<Self>, GameKitError> {
        Self::load(&[])
    }

    /// Loads the previous occurrence for a recurring leaderboard.
    pub fn load_previous_occurrence(&self) -> Result<Option<Self>, GameKitError> {
        let leaderboard_id =
            private::cstring_from_str(&self.base_leaderboard_id, "leaderboard identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_leaderboard_load_previous_occurrence_json(
                leaderboard_id.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payload: Option<LeaderboardPayload> =
                private::parse_json_ptr(out_json, "previous leaderboard occurrence")?;
            Ok(payload.map(Self::from_payload))
        }
    }

    /// Submits a score to multiple leaderboards for the local player.
    pub fn submit_score(
        score: i64,
        context: u64,
        leaderboard_ids: &[&str],
    ) -> Result<(), GameKitError> {
        let ids_json = private::json_cstring(leaderboard_ids, "leaderboard identifiers")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_leaderboard_submit_score(
                score,
                context,
                ids_json.as_ptr(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Submits a score to this leaderboard for the local player.
    pub fn submit_local_score(&self, score: i64, context: u64) -> Result<(), GameKitError> {
        let leaderboard_id =
            private::cstring_from_str(&self.base_leaderboard_id, "leaderboard identifier")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_leaderboard_submit_score_for_id(
                score,
                context,
                leaderboard_id.as_ptr(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    fn from_payload(payload: LeaderboardPayload) -> Self {
        Self {
            base_leaderboard_id: payload.base_leaderboard_id,
            title: payload.title,
            group_identifier: payload.group_identifier,
            leaderboard_type: match payload.leaderboard_type.as_str() {
                "recurring" => LeaderboardType::Recurring,
                _ => LeaderboardType::Classic,
            },
            start_date: payload.start_date,
            next_start_date: payload.next_start_date,
            duration_seconds: payload.duration_seconds,
        }
    }
}
