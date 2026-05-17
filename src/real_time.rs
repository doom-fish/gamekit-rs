use core::ffi::{c_char, c_void};
use std::collections::BTreeMap;
use std::ffi::CStr;

use serde::{Deserialize, Serialize};

use crate::{ffi, private, GameKitError, InviteRecipientResponse, Match, Player};

/// High-level real-time match type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    PeerToPeer,
    Hosted,
    TurnBased,
}

/// JSON-compatible matchmaking properties returned by rule-based matchmaking APIs.
pub type MatchProperties = BTreeMap<String, serde_json::Value>;

/// Properties associated with a specific matched player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchedPlayerProperties {
    pub player: Player,
    pub properties: MatchProperties,
}

/// Rule-based matchmaking results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchedPlayers {
    pub properties: Option<MatchProperties>,
    pub players: Vec<Player>,
    pub player_properties: Vec<MatchedPlayerProperties>,
}

/// Request parameters for matchmaking.
#[derive(Debug, Clone)]
pub struct MatchRequest {
    pub min_players: u32,
    pub max_players: u32,
    pub player_group: u32,
    pub player_attributes: u32,
    pub recipient_ids: Vec<String>,
    pub invite_message: Option<String>,
    pub default_number_of_players: Option<u32>,
}

impl Default for MatchRequest {
    fn default() -> Self {
        Self {
            min_players: 2,
            max_players: 4,
            player_group: 0,
            player_attributes: 0,
            recipient_ids: Vec::new(),
            invite_message: None,
            default_number_of_players: None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MatchRequestPayload<'a> {
    min_players: u32,
    max_players: u32,
    player_group: u32,
    player_attributes: u32,
    recipient_ids: &'a [String],
    invite_message: Option<&'a str>,
    default_number_of_players: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MatchedPlayerPropertiesPayload {
    player: Player,
    properties_json: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MatchedPlayersPayload {
    properties_json: Option<String>,
    players: Vec<Player>,
    player_properties: Vec<MatchedPlayerPropertiesPayload>,
}

/// Provides access to the Game Center real-time matchmaking service.
#[derive(Debug, Clone, Copy, Default)]
pub struct Matchmaker;

impl Matchmaker {
    /// Returns the shared matchmaker instance.
    #[must_use]
    pub const fn shared() -> Self {
        Self
    }

    /// Finds a real-time peer-to-peer match for the given request.
    pub fn find_match(&self, request: &MatchRequest) -> Result<Match, GameKitError> {
        let request_json = request_json(request)?;

        unsafe {
            let mut out_match_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_find_match_json(
                request_json.as_ptr(),
                None,
                std::ptr::null_mut(),
                &mut out_match_ptr,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(Match::from_raw(out_match_ptr))
        }
    }

    /// Finds a real-time peer-to-peer match while observing invite-recipient responses.
    pub fn find_match_with_recipient_responses<
        F: FnMut(Player, InviteRecipientResponse) + Send + 'static,
    >(
        &self,
        request: &MatchRequest,
        handler: F,
    ) -> Result<Match, GameKitError> {
        let request_json = request_json(request)?;
        let boxed: Box<dyn FnMut(Player, InviteRecipientResponse) + Send + 'static> =
            Box::new(handler);
        let handler_ptr = Box::into_raw(Box::new(boxed));

        unsafe {
            let mut out_match_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_find_match_json(
                request_json.as_ptr(),
                Some(invite_recipient_response_trampoline),
                handler_ptr.cast(),
                &mut out_match_ptr,
                &mut out_error,
            );
            drop(Box::from_raw(handler_ptr));
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(Match::from_raw(out_match_ptr))
        }
    }

    /// Finds players for a hosted real-time match.
    pub fn find_hosted_players(&self, request: &MatchRequest) -> Result<Vec<Player>, GameKitError> {
        let request_json = request_json(request)?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_find_hosted_players_json(
                request_json.as_ptr(),
                None,
                std::ptr::null_mut(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "hosted players")
        }
    }

    /// Finds hosted-match players while observing invite-recipient responses.
    pub fn find_hosted_players_with_recipient_responses<
        F: FnMut(Player, InviteRecipientResponse) + Send + 'static,
    >(
        &self,
        request: &MatchRequest,
        handler: F,
    ) -> Result<Vec<Player>, GameKitError> {
        let request_json = request_json(request)?;
        let boxed: Box<dyn FnMut(Player, InviteRecipientResponse) + Send + 'static> =
            Box::new(handler);
        let handler_ptr = Box::into_raw(Box::new(boxed));

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_find_hosted_players_json(
                request_json.as_ptr(),
                Some(invite_recipient_response_trampoline),
                handler_ptr.cast(),
                &mut out_json,
                &mut out_error,
            );
            drop(Box::from_raw(handler_ptr));
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "hosted players")
        }
    }

    /// Finds rule-based matched players for the given request.
    pub fn find_matched_players(
        &self,
        request: &MatchRequest,
    ) -> Result<MatchedPlayers, GameKitError> {
        let request_json = request_json(request)?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_find_matched_players_json(
                request_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payload: MatchedPlayersPayload =
                private::parse_json_ptr(out_json, "matched players")?;
            MatchedPlayers::from_payload(payload)
        }
    }

    /// Adds players to an existing peer-to-peer match.
    pub fn add_players_to_match(
        &self,
        existing_match: &Match,
        request: &MatchRequest,
    ) -> Result<(), GameKitError> {
        let request_json = request_json(request)?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_add_players_to_match(
                existing_match.ptr,
                request_json.as_ptr(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Cancels any pending matchmaking or invites.
    pub fn cancel(&self) {
        unsafe {
            ffi::gk_matchmaker_cancel();
        }
    }

    /// Finishes programmatic matchmaking for an existing match.
    pub fn finish_matchmaking(&self, existing_match: &Match) {
        unsafe {
            ffi::gk_matchmaker_finish(existing_match.ptr);
        }
    }

    /// Queries recent activity for a specific player group.
    pub fn query_player_group_activity(&self, player_group: usize) -> Result<i64, GameKitError> {
        unsafe {
            let mut out_activity = 0_i64;
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_query_player_group_activity(
                player_group,
                &mut out_activity,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(out_activity)
        }
    }

    /// Queries aggregate Game Center activity for the game.
    pub fn query_activity(&self) -> Result<i64, GameKitError> {
        unsafe {
            let mut out_activity = 0_i64;
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_query_activity(&mut out_activity, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(out_activity)
        }
    }

    /// Returns the maximum number of players allowed for the supplied match type.
    #[must_use]
    pub fn max_players_allowed(match_type: MatchType) -> usize {
        unsafe { ffi::gk_matchmaker_max_players_allowed(match_type_to_i32(match_type)) }
    }
}

impl<'a> MatchRequestPayload<'a> {
    pub(crate) fn from_request(request: &'a MatchRequest) -> Self {
        Self {
            min_players: request.min_players,
            max_players: request.max_players,
            player_group: request.player_group,
            player_attributes: request.player_attributes,
            recipient_ids: &request.recipient_ids,
            invite_message: request.invite_message.as_deref(),
            default_number_of_players: request
                .default_number_of_players
                .unwrap_or(request.max_players),
        }
    }
}

impl MatchedPlayers {
    fn from_payload(payload: MatchedPlayersPayload) -> Result<Self, GameKitError> {
        Ok(Self {
            properties: parse_match_properties(
                payload.properties_json,
                "matched-player properties",
            )?,
            players: payload.players,
            player_properties: payload
                .player_properties
                .into_iter()
                .map(MatchedPlayerProperties::from_payload)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl MatchedPlayerProperties {
    fn from_payload(payload: MatchedPlayerPropertiesPayload) -> Result<Self, GameKitError> {
        Ok(Self {
            player: payload.player,
            properties: parse_match_properties(payload.properties_json, "matched-player override")?
                .unwrap_or_default(),
        })
    }
}

pub(crate) fn request_json(request: &MatchRequest) -> Result<std::ffi::CString, GameKitError> {
    let payload = MatchRequestPayload::from_request(request);
    private::json_cstring(&payload, "match request")
}

fn parse_match_properties(
    json: Option<String>,
    context: &str,
) -> Result<Option<MatchProperties>, GameKitError> {
    json.map(|json| {
        serde_json::from_str::<MatchProperties>(&json).map_err(|error| {
            GameKitError::Unknown(format!(
                "failed to parse {context} JSON: {error}; payload={json}"
            ))
        })
    })
    .transpose()
}

unsafe extern "C" fn invite_recipient_response_trampoline(
    refcon: *mut c_void,
    player_json: *const c_char,
    response: i32,
) {
    if player_json.is_null() {
        return;
    }

    let handler =
        &mut *(refcon.cast::<Box<dyn FnMut(Player, InviteRecipientResponse) + Send + 'static>>());
    if let Ok(player_str) = CStr::from_ptr(player_json).to_str() {
        if let Ok(player) = serde_json::from_str::<Player>(player_str) {
            handler(player, InviteRecipientResponse::from_raw(response));
        }
    }
}

const fn match_type_to_i32(match_type: MatchType) -> i32 {
    match match_type {
        MatchType::PeerToPeer => 0,
        MatchType::Hosted => 1,
        MatchType::TurnBased => 2,
    }
}
