#![allow(clippy::too_many_lines)]

use core::ffi::c_char;

use serde::{Deserialize, Serialize};

use crate::{ffi, private, Achievement, GameKitError, Player, Score};

/// Overall state of a turn-based match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnBasedMatchStatus {
    Unknown,
    Open,
    Ended,
    Matching,
}

/// State of an individual participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnBasedParticipantStatus {
    Unknown,
    Invited,
    Declined,
    Matching,
    Active,
    Done,
}

/// Match outcome for a completed participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnBasedMatchOutcome {
    None,
    Quit,
    Won,
    Lost,
    Tied,
    TimeExpired,
    First,
    Second,
    Third,
    Fourth,
    Custom(i32),
}

/// Status of a turn-based exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnBasedExchangeStatus {
    Unknown,
    Active,
    Complete,
    Resolved,
    Canceled,
}

/// Request parameters for a turn-based match.
#[derive(Debug, Clone)]
pub struct TurnBasedMatchRequest {
    pub min_players: u32,
    pub max_players: u32,
    pub player_group: u32,
    pub player_attributes: u32,
    pub recipient_ids: Vec<String>,
}

impl Default for TurnBasedMatchRequest {
    fn default() -> Self {
        Self {
            min_players: 2,
            max_players: 2,
            player_group: 0,
            player_attributes: 0,
            recipient_ids: Vec::new(),
        }
    }
}

/// Snapshot of a turn-based participant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnBasedParticipant {
    pub index: usize,
    pub player: Option<Player>,
    pub last_turn_date: Option<String>,
    pub status: TurnBasedParticipantStatus,
    pub match_outcome: TurnBasedMatchOutcome,
    pub timeout_date: Option<String>,
}

/// Snapshot of a turn-based exchange reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnBasedExchangeReply {
    pub recipient_index: Option<usize>,
    pub message: Option<String>,
    pub data_len: usize,
    pub reply_date: Option<String>,
}

/// Snapshot of a turn-based exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnBasedExchange {
    pub index: usize,
    pub exchange_id: Option<String>,
    pub sender_index: Option<usize>,
    pub recipient_indices: Vec<usize>,
    pub status: TurnBasedExchangeStatus,
    pub message: Option<String>,
    pub data_len: usize,
    pub send_date: Option<String>,
    pub timeout_date: Option<String>,
    pub completion_date: Option<String>,
    pub replies: Vec<TurnBasedExchangeReply>,
}

/// Snapshot of a turn-based match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnBasedMatch {
    pub match_id: String,
    pub creation_date: Option<String>,
    pub participants: Vec<TurnBasedParticipant>,
    pub status: TurnBasedMatchStatus,
    pub current_participant_index: Option<usize>,
    pub message: Option<String>,
    pub match_data_len: usize,
    pub match_data_maximum_size: usize,
    pub exchanges: Vec<TurnBasedExchange>,
    pub active_exchange_indices: Vec<usize>,
    pub completed_exchange_indices: Vec<usize>,
    pub exchange_data_maximum_size: usize,
    pub exchange_max_initiated_exchanges_per_player: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TurnBasedMatchRequestPayload<'a> {
    min_players: u32,
    max_players: u32,
    player_group: u32,
    player_attributes: u32,
    recipient_ids: &'a [String],
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TurnBasedParticipantPayload {
    index: usize,
    player: Option<Player>,
    last_turn_date: Option<String>,
    status: i32,
    match_outcome: i32,
    timeout_date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TurnBasedExchangeReplyPayload {
    recipient_index: Option<usize>,
    message: Option<String>,
    data_len: usize,
    reply_date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TurnBasedExchangePayload {
    index: usize,
    exchange_id: Option<String>,
    sender_index: Option<usize>,
    recipient_indices: Vec<usize>,
    status: i32,
    message: Option<String>,
    data_len: usize,
    send_date: Option<String>,
    timeout_date: Option<String>,
    completion_date: Option<String>,
    replies: Vec<TurnBasedExchangeReplyPayload>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TurnBasedMatchPayload {
    match_id: String,
    creation_date: Option<String>,
    participants: Vec<TurnBasedParticipantPayload>,
    status: i32,
    current_participant_index: Option<usize>,
    message: Option<String>,
    match_data_len: usize,
    match_data_maximum_size: usize,
    exchanges: Vec<TurnBasedExchangePayload>,
    active_exchange_indices: Vec<usize>,
    completed_exchange_indices: Vec<usize>,
    exchange_data_maximum_size: usize,
    exchange_max_initiated_exchanges_per_player: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinaryPayload {
    data_base64: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AchievementInputPayload<'a> {
    identifier: &'a str,
    percent_complete: f64,
    shows_completion_banner: bool,
    player_game_id: Option<&'a str>,
}

impl TurnBasedMatch {
    /// Finds a new turn-based match for the supplied request.
    pub fn find(request: &TurnBasedMatchRequest) -> Result<Self, GameKitError> {
        let payload = TurnBasedMatchRequestPayload::from_request(request);
        let request_json = private::json_cstring(&payload, "turn-based match request")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_find_match_json(
                request_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payload: TurnBasedMatchPayload =
                private::parse_json_ptr(out_json, "turn-based match")?;
            Ok(Self::from_payload(payload))
        }
    }

    /// Loads every turn-based match for the local player.
    pub fn load_matches() -> Result<Vec<Self>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_load_matches_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payloads: Vec<TurnBasedMatchPayload> =
                private::parse_json_ptr(out_json, "turn-based matches")?;
            Ok(payloads.into_iter().map(Self::from_payload).collect())
        }
    }

    /// Loads a match by identifier.
    pub fn load(match_id: &str) -> Result<Self, GameKitError> {
        let match_id = private::cstring_from_str(match_id, "match identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_load_match_json(
                match_id.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payload: TurnBasedMatchPayload =
                private::parse_json_ptr(out_json, "turn-based match")?;
            Ok(Self::from_payload(payload))
        }
    }

    /// Requests a rematch with the same participants.
    pub fn rematch(&self) -> Result<Self, GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_rematch_json(
                match_id.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payload: TurnBasedMatchPayload =
                private::parse_json_ptr(out_json, "turn-based rematch")?;
            Ok(Self::from_payload(payload))
        }
    }

    /// Accepts a pending invite and returns the updated match snapshot.
    pub fn accept_invite(&self) -> Result<Self, GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_accept_invite_json(
                match_id.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payload: TurnBasedMatchPayload =
                private::parse_json_ptr(out_json, "accepted turn-based match")?;
            Ok(Self::from_payload(payload))
        }
    }

    /// Declines a pending invite for this match.
    pub fn decline_invite(&self) -> Result<(), GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_decline_invite(match_id.as_ptr(), &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Removes the match from the local player's list.
    pub fn remove(&self) -> Result<(), GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_remove(match_id.as_ptr(), &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Loads the raw match data for this turn-based match.
    pub fn load_match_data(&self) -> Result<Vec<u8>, GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_load_match_data_json(
                match_id.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payload: BinaryPayload = private::parse_json_ptr(out_json, "turn-based match data")?;
            private::decode_base64(&payload.data_base64, "turn-based match")
        }
    }

    /// Saves match data without ending the current turn.
    pub fn save_current_turn(&self, data: &[u8]) -> Result<(), GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_save_current_turn(
                match_id.as_ptr(),
                data.as_ptr(),
                data.len(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Ends the current turn and advances to the supplied participants.
    pub fn end_turn(
        &self,
        next_participant_indices: &[usize],
        timeout_seconds: f64,
        data: &[u8],
    ) -> Result<(), GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;
        let next_indices = private::json_cstring(next_participant_indices, "participant indices")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_end_turn(
                match_id.as_ptr(),
                next_indices.as_ptr(),
                timeout_seconds,
                data.as_ptr(),
                data.len(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Ends the current player's turn while quitting the match.
    pub fn participant_quit_in_turn(
        &self,
        outcome: TurnBasedMatchOutcome,
        next_participant_indices: &[usize],
        timeout_seconds: f64,
        data: &[u8],
    ) -> Result<(), GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;
        let next_indices = private::json_cstring(next_participant_indices, "participant indices")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_participant_quit_in_turn(
                match_id.as_ptr(),
                outcome.to_raw(),
                next_indices.as_ptr(),
                timeout_seconds,
                data.as_ptr(),
                data.len(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Quits the match when it isn't the local player's turn.
    pub fn participant_quit_out_of_turn(
        &self,
        outcome: TurnBasedMatchOutcome,
    ) -> Result<(), GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_participant_quit_out_of_turn(
                match_id.as_ptr(),
                outcome.to_raw(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Ends the match and optionally submits scores and achievements.
    pub fn end_match_in_turn(
        &self,
        data: &[u8],
        scores: &[Score],
        achievements: &[Achievement],
    ) -> Result<(), GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;
        let scores_json = if scores.is_empty() {
            None
        } else {
            Some(private::json_cstring(scores, "scores")?)
        };
        let achievement_payloads: Vec<AchievementInputPayload<'_>> = achievements
            .iter()
            .map(|achievement| AchievementInputPayload {
                identifier: &achievement.identifier,
                percent_complete: achievement.percent_complete,
                shows_completion_banner: achievement.shows_completion_banner,
                player_game_id: achievement.player.as_ref().map(|player| player.game_player_id.as_str()),
            })
            .collect();
        let achievements_json = if achievement_payloads.is_empty() {
            None
        } else {
            Some(private::json_cstring(&achievement_payloads, "achievements")?)
        };

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_end_match_in_turn(
                match_id.as_ptr(),
                data.as_ptr(),
                data.len(),
                scores_json.as_ref().map_or(std::ptr::null(), |json| json.as_ptr()),
                achievements_json
                    .as_ref()
                    .map_or(std::ptr::null(), |json| json.as_ptr()),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Saves merged match data while resolving completed exchanges.
    pub fn save_merged_match_data(
        &self,
        data: &[u8],
        resolved_exchange_indices: &[usize],
    ) -> Result<(), GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;
        let resolved_indices = private::json_cstring(resolved_exchange_indices, "exchange indices")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_save_merged_match_data(
                match_id.as_ptr(),
                data.as_ptr(),
                data.len(),
                resolved_indices.as_ptr(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Sends an exchange request and returns the created exchange snapshot.
    pub fn send_exchange(
        &self,
        participant_indices: &[usize],
        data: &[u8],
        message_key: &str,
        arguments: &[&str],
        timeout_seconds: f64,
    ) -> Result<TurnBasedExchange, GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;
        let participant_indices = private::json_cstring(participant_indices, "participant indices")?;
        let message_key = private::cstring_from_str(message_key, "message key")?;
        let arguments = private::json_cstring(arguments, "message arguments")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_send_exchange_json(
                match_id.as_ptr(),
                participant_indices.as_ptr(),
                data.as_ptr(),
                data.len(),
                message_key.as_ptr(),
                arguments.as_ptr(),
                timeout_seconds,
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payload: TurnBasedExchangePayload =
                private::parse_json_ptr(out_json, "turn-based exchange")?;
            Ok(TurnBasedExchange::from_payload(payload))
        }
    }

    /// Cancels an exchange with a localizable message.
    pub fn cancel_exchange(
        &self,
        exchange_index: usize,
        message_key: &str,
        arguments: &[&str],
    ) -> Result<(), GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;
        let message_key = private::cstring_from_str(message_key, "message key")?;
        let arguments = private::json_cstring(arguments, "message arguments")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_cancel_exchange(
                match_id.as_ptr(),
                exchange_index,
                message_key.as_ptr(),
                arguments.as_ptr(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Replies to an exchange with a localizable message and optional data.
    pub fn reply_exchange(
        &self,
        exchange_index: usize,
        message_key: &str,
        arguments: &[&str],
        data: &[u8],
    ) -> Result<(), GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;
        let message_key = private::cstring_from_str(message_key, "message key")?;
        let arguments = private::json_cstring(arguments, "message arguments")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_reply_exchange(
                match_id.as_ptr(),
                exchange_index,
                message_key.as_ptr(),
                arguments.as_ptr(),
                data.as_ptr(),
                data.len(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Sends a reminder to one or more participants.
    pub fn send_reminder(
        &self,
        participant_indices: &[usize],
        message_key: &str,
        arguments: &[&str],
    ) -> Result<(), GameKitError> {
        let match_id = private::cstring_from_str(&self.match_id, "match identifier")?;
        let participant_indices = private::json_cstring(participant_indices, "participant indices")?;
        let message_key = private::cstring_from_str(message_key, "message key")?;
        let arguments = private::json_cstring(arguments, "message arguments")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_send_reminder(
                match_id.as_ptr(),
                participant_indices.as_ptr(),
                message_key.as_ptr(),
                arguments.as_ptr(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    fn from_payload(payload: TurnBasedMatchPayload) -> Self {
        Self {
            match_id: payload.match_id,
            creation_date: payload.creation_date,
            participants: payload
                .participants
                .into_iter()
                .map(TurnBasedParticipant::from_payload)
                .collect(),
            status: TurnBasedMatchStatus::from_raw(payload.status),
            current_participant_index: payload.current_participant_index,
            message: payload.message,
            match_data_len: payload.match_data_len,
            match_data_maximum_size: payload.match_data_maximum_size,
            exchanges: payload
                .exchanges
                .into_iter()
                .map(TurnBasedExchange::from_payload)
                .collect(),
            active_exchange_indices: payload.active_exchange_indices,
            completed_exchange_indices: payload.completed_exchange_indices,
            exchange_data_maximum_size: payload.exchange_data_maximum_size,
            exchange_max_initiated_exchanges_per_player: payload
                .exchange_max_initiated_exchanges_per_player,
        }
    }
}

impl TurnBasedParticipant {
    fn from_payload(payload: TurnBasedParticipantPayload) -> Self {
        Self {
            index: payload.index,
            player: payload.player,
            last_turn_date: payload.last_turn_date,
            status: TurnBasedParticipantStatus::from_raw(payload.status),
            match_outcome: TurnBasedMatchOutcome::from_raw(payload.match_outcome),
            timeout_date: payload.timeout_date,
        }
    }
}

impl TurnBasedExchange {
    fn from_payload(payload: TurnBasedExchangePayload) -> Self {
        Self {
            index: payload.index,
            exchange_id: payload.exchange_id,
            sender_index: payload.sender_index,
            recipient_indices: payload.recipient_indices,
            status: TurnBasedExchangeStatus::from_raw(payload.status),
            message: payload.message,
            data_len: payload.data_len,
            send_date: payload.send_date,
            timeout_date: payload.timeout_date,
            completion_date: payload.completion_date,
            replies: payload
                .replies
                .into_iter()
                .map(TurnBasedExchangeReply::from_payload)
                .collect(),
        }
    }
}

impl TurnBasedExchangeReply {
    fn from_payload(payload: TurnBasedExchangeReplyPayload) -> Self {
        Self {
            recipient_index: payload.recipient_index,
            message: payload.message,
            data_len: payload.data_len,
            reply_date: payload.reply_date,
        }
    }
}

impl TurnBasedMatchStatus {
    const fn from_raw(value: i32) -> Self {
        match value {
            1 => Self::Open,
            2 => Self::Ended,
            3 => Self::Matching,
            _ => Self::Unknown,
        }
    }
}

impl TurnBasedParticipantStatus {
    const fn from_raw(value: i32) -> Self {
        match value {
            1 => Self::Invited,
            2 => Self::Declined,
            3 => Self::Matching,
            4 => Self::Active,
            5 => Self::Done,
            _ => Self::Unknown,
        }
    }
}

impl TurnBasedMatchOutcome {
    const fn from_raw(value: i32) -> Self {
        match value {
            1 => Self::Quit,
            2 => Self::Won,
            3 => Self::Lost,
            4 => Self::Tied,
            5 => Self::TimeExpired,
            6 => Self::First,
            7 => Self::Second,
            8 => Self::Third,
            9 => Self::Fourth,
            0 => Self::None,
            other => Self::Custom(other),
        }
    }

    const fn to_raw(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Quit => 1,
            Self::Won => 2,
            Self::Lost => 3,
            Self::Tied => 4,
            Self::TimeExpired => 5,
            Self::First => 6,
            Self::Second => 7,
            Self::Third => 8,
            Self::Fourth => 9,
            Self::Custom(value) => value,
        }
    }
}

impl TurnBasedExchangeStatus {
    const fn from_raw(value: i32) -> Self {
        match value {
            1 => Self::Active,
            2 => Self::Complete,
            3 => Self::Resolved,
            4 => Self::Canceled,
            _ => Self::Unknown,
        }
    }
}

impl<'a> TurnBasedMatchRequestPayload<'a> {
    fn from_request(request: &'a TurnBasedMatchRequest) -> Self {
        Self {
            min_players: request.min_players,
            max_players: request.max_players,
            player_group: request.player_group,
            player_attributes: request.player_attributes,
            recipient_ids: &request.recipient_ids,
        }
    }
}
