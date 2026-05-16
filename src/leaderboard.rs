use core::ffi::c_char;
use std::ops::Range;

use serde::Deserialize;

use crate::{ffi, private, GameKitError, Player};

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
#[derive(Debug, Clone)]
pub struct Leaderboard {
    pub base_leaderboard_id: String,
    pub title: Option<String>,
    pub leaderboard_type: LeaderboardType,
}

/// A single entry in a leaderboard.
#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub score: i64,
    pub formatted_score: String,
    pub context: u64,
    pub date: String,
    pub player: Player,
}

/// Result of loading leaderboard entries.
#[derive(Debug, Clone)]
pub struct LoadEntriesResult {
    pub local_player_entry: Option<LeaderboardEntry>,
    pub entries: Vec<LeaderboardEntry>,
    pub total_player_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LeaderboardPayload {
    base_leaderboard_id: String,
    title: Option<String>,
    leaderboard_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LeaderboardEntryPayload {
    rank: i64,
    score: i64,
    formatted_score: String,
    context: u64,
    date: String,
    player: Player,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoadEntriesPayload {
    local_player_entry: Option<LeaderboardEntryPayload>,
    entries: Vec<LeaderboardEntryPayload>,
    total_player_count: i64,
}

impl Leaderboard {
    /// Loads leaderboards by their IDs.
    pub fn load<I, S>(ids: I) -> Result<Vec<Self>, GameKitError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let ids_vec: Vec<String> = ids.into_iter().map(|s| s.as_ref().to_owned()).collect();
        let ids_refs: Vec<&str> = ids_vec.iter().map(String::as_str).collect();
        let ids_json = private::json_cstring(&ids_refs, "leaderboard IDs")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_leaderboard_load_json(
                ids_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            );

            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payloads: Vec<LeaderboardPayload> =
                private::parse_json_ptr(out_json, "leaderboards")?;

            Ok(payloads
                .into_iter()
                .map(|p| Self {
                    base_leaderboard_id: p.base_leaderboard_id,
                    title: p.title,
                    leaderboard_type: match p.leaderboard_type.as_str() {
                        "recurring" => LeaderboardType::Recurring,
                        _ => LeaderboardType::Classic,
                    },
                })
                .collect())
        }
    }

    /// Submits a score to multiple leaderboards.
    pub fn submit_score(
        score: i64,
        context: u64,
        leaderboard_ids: &[&str],
    ) -> Result<(), GameKitError> {
        let ids_json = private::json_cstring(&leaderboard_ids, "leaderboard IDs")?;

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

    /// Loads entries for this leaderboard.
    pub fn load_entries(
        &self,
        player_scope: PlayerScope,
        time_scope: TimeScope,
        range: Range<usize>,
    ) -> Result<LoadEntriesResult, GameKitError> {
        let id_cstring = private::cstring_from_str(&self.base_leaderboard_id, "leaderboard ID")?;

        let player_scope_i32 = match player_scope {
            PlayerScope::Global => 0,
            PlayerScope::FriendsOnly => 1,
        };

        let time_scope_i32 = match time_scope {
            TimeScope::Today => 0,
            TimeScope::Week => 1,
            TimeScope::AllTime => 2,
        };

        let range_location = range.start + 1;
        let range_length = range.end.saturating_sub(range.start);

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_leaderboard_load_entries_json(
                id_cstring.as_ptr(),
                player_scope_i32,
                time_scope_i32,
                range_location,
                range_length,
                &mut out_json,
                &mut out_error,
            );

            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payload: LoadEntriesPayload =
                private::parse_json_ptr(out_json, "load entries result")?;

            Ok(LoadEntriesResult {
                local_player_entry: payload.local_player_entry.map(|e| LeaderboardEntry {
                    rank: e.rank,
                    score: e.score,
                    formatted_score: e.formatted_score,
                    context: e.context,
                    date: e.date,
                    player: e.player,
                }),
                entries: payload
                    .entries
                    .into_iter()
                    .map(|e| LeaderboardEntry {
                        rank: e.rank,
                        score: e.score,
                        formatted_score: e.formatted_score,
                        context: e.context,
                        date: e.date,
                        player: e.player,
                    })
                    .collect(),
                total_player_count: payload.total_player_count,
            })
        }
    }
}
