use core::ffi::c_char;
use std::ops::Range;

use serde::Deserialize;

use crate::leaderboard::{Leaderboard, PlayerScope, TimeScope};
use crate::{ffi, private, GameKitError, Player};

/// A single entry in a leaderboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub score: i64,
    pub formatted_score: String,
    pub context: u64,
    pub date: String,
    pub player: Player,
}

/// Result of loading leaderboard entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadEntriesResult {
    pub local_player_entry: Option<LeaderboardEntry>,
    pub entries: Vec<LeaderboardEntry>,
    pub total_player_count: i64,
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
    /// Loads entries for this leaderboard.
    pub fn load_entries(
        &self,
        player_scope: PlayerScope,
        time_scope: TimeScope,
        range: Range<usize>,
    ) -> Result<LoadEntriesResult, GameKitError> {
        let leaderboard_id =
            private::cstring_from_str(&self.base_leaderboard_id, "leaderboard identifier")?;
        let (range_location, range_length) = to_ns_range(range);

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_leaderboard_load_entries_json(
                leaderboard_id.as_ptr(),
                player_scope_to_i32(player_scope),
                time_scope_to_i32(time_scope),
                range_location,
                range_length,
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payload: LoadEntriesPayload =
                private::parse_json_ptr(out_json, "leaderboard entries")?;
            Ok(LoadEntriesResult {
                local_player_entry: payload.local_player_entry.map(LeaderboardEntry::from_payload),
                entries: payload
                    .entries
                    .into_iter()
                    .map(LeaderboardEntry::from_payload)
                    .collect(),
                total_player_count: payload.total_player_count,
            })
        }
    }

    /// Loads entries for specific players identified by scoped player identifiers.
    pub fn load_entries_for_players(
        &self,
        player_ids: &[&str],
        time_scope: TimeScope,
    ) -> Result<LoadEntriesResult, GameKitError> {
        let leaderboard_id =
            private::cstring_from_str(&self.base_leaderboard_id, "leaderboard identifier")?;
        let player_ids_json = private::json_cstring(player_ids, "player identifiers")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_leaderboard_load_entries_for_players_json(
                leaderboard_id.as_ptr(),
                player_ids_json.as_ptr(),
                time_scope_to_i32(time_scope),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payload: LoadEntriesPayload =
                private::parse_json_ptr(out_json, "leaderboard entries for players")?;
            Ok(LoadEntriesResult {
                local_player_entry: payload.local_player_entry.map(LeaderboardEntry::from_payload),
                entries: payload
                    .entries
                    .into_iter()
                    .map(LeaderboardEntry::from_payload)
                    .collect(),
                total_player_count: payload.total_player_count,
            })
        }
    }
}

impl LeaderboardEntry {
    fn from_payload(payload: LeaderboardEntryPayload) -> Self {
        Self {
            rank: payload.rank,
            score: payload.score,
            formatted_score: payload.formatted_score,
            context: payload.context,
            date: payload.date,
            player: payload.player,
        }
    }
}

const fn to_ns_range(range: Range<usize>) -> (usize, usize) {
    let location = range.start.saturating_add(1);
    let length = range.end.saturating_sub(range.start);
    (location, length)
}

const fn player_scope_to_i32(scope: PlayerScope) -> i32 {
    match scope {
        PlayerScope::Global => 0,
        PlayerScope::FriendsOnly => 1,
    }
}

const fn time_scope_to_i32(scope: TimeScope) -> i32 {
    match scope {
        TimeScope::Today => 0,
        TimeScope::Week => 1,
        TimeScope::AllTime => 2,
    }
}
