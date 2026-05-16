use core::ffi::{c_char, c_void};
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    ffi, private, Achievement, AchievementDescription, GameKitError, Leaderboard, Match,
    MatchRequest, Player, Score,
};

/// Play style metadata for a Game Center activity definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameActivityPlayStyle {
    Unspecified,
    Synchronous,
    Asynchronous,
    Unknown(i32),
}

/// Lifecycle state for a game activity instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameActivityState {
    Initialized,
    Active,
    Paused,
    Ended,
    Unknown(i32),
}

/// A developer-defined Game Center activity definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameActivityDefinition {
    pub identifier: String,
    pub group_identifier: Option<String>,
    pub title: String,
    pub details: Option<String>,
    pub default_properties: BTreeMap<String, String>,
    pub fallback_url: Option<String>,
    pub supports_party_code: bool,
    pub max_players: Option<u32>,
    pub min_players: Option<u32>,
    pub supports_unlimited_players: bool,
    pub play_style: GameActivityPlayStyle,
    pub release_state: String,
}

/// A snapshot of a running or pending game activity instance.
#[derive(Debug, Clone, PartialEq)]
pub struct GameActivitySnapshot {
    pub identifier: String,
    pub activity_definition: GameActivityDefinition,
    pub properties: BTreeMap<String, String>,
    pub state: GameActivityState,
    pub party_code: Option<String>,
    pub party_url: Option<String>,
    pub creation_date: String,
    pub start_date: Option<String>,
    pub last_resume_date: Option<String>,
    pub end_date: Option<String>,
    pub duration_seconds: f64,
    pub achievements: Vec<Achievement>,
    pub leaderboard_scores: Vec<Score>,
}

/// A live `GKGameActivity` instance.
pub struct GameActivity {
    ptr: *mut c_void,
}

unsafe impl Send for GameActivity {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GameActivityDefinitionPayload {
    pub(crate) identifier: String,
    pub(crate) group_identifier: Option<String>,
    pub(crate) title: String,
    pub(crate) details: Option<String>,
    pub(crate) default_properties: BTreeMap<String, String>,
    #[serde(rename = "fallbackURL")]
    pub(crate) fallback_url: Option<String>,
    pub(crate) supports_party_code: bool,
    pub(crate) max_players: Option<u32>,
    pub(crate) min_players: Option<u32>,
    pub(crate) supports_unlimited_players: bool,
    pub(crate) play_style: i32,
    pub(crate) release_state: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameActivityAchievementPayload {
    identifier: String,
    percent_complete: f64,
    is_completed: bool,
    last_reported_date: Option<String>,
    shows_completion_banner: bool,
    player: Option<Player>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GameActivityScorePayload {
    #[serde(rename = "leaderboardID")]
    leaderboard_id: String,
    value: i64,
    context: u64,
    #[serde(rename = "playerGameID")]
    player_game_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GameActivitySnapshotPayload {
    identifier: String,
    #[serde(rename = "activityDefinition")]
    activity_definition: GameActivityDefinitionPayload,
    properties: BTreeMap<String, String>,
    state: i32,
    #[serde(rename = "partyCode")]
    party_code: Option<String>,
    #[serde(rename = "partyURL")]
    party_url: Option<String>,
    #[serde(rename = "creationDate")]
    creation_date: String,
    #[serde(rename = "startDate")]
    start_date: Option<String>,
    #[serde(rename = "lastResumeDate")]
    last_resume_date: Option<String>,
    #[serde(rename = "endDate")]
    end_date: Option<String>,
    #[serde(rename = "durationSeconds")]
    duration_seconds: f64,
    achievements: Vec<GameActivityAchievementPayload>,
    #[serde(rename = "leaderboardScores")]
    leaderboard_scores: Vec<GameActivityScorePayload>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MatchRequestPayload {
    min_players: u32,
    max_players: u32,
    player_group: u32,
    player_attributes: u32,
    recipient_ids: Vec<String>,
    invite_message: Option<String>,
    default_number_of_players: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinaryPayload {
    data_base64: String,
}

#[derive(Debug, Serialize)]
struct GameActivityScoreInputPayload<'a> {
    #[serde(rename = "leaderboardID")]
    leaderboard_id: &'a str,
    value: i64,
    context: u64,
    #[serde(rename = "playerGameID")]
    player_game_id: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct GameActivityAchievementInputPayload<'a> {
    identifier: &'a str,
    #[serde(rename = "percentComplete")]
    percent_complete: f64,
    #[serde(rename = "showsCompletionBanner")]
    shows_completion_banner: bool,
    #[serde(rename = "playerGameID")]
    player_game_id: Option<&'a str>,
}

impl GameActivityDefinition {
    /// Loads activity definitions for the current game.
    pub fn load(ids: &[&str]) -> Result<Vec<Self>, GameKitError> {
        let ids_json = if ids.is_empty() {
            None
        } else {
            Some(private::json_cstring(
                ids,
                "game activity definition identifiers",
            )?)
        };

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_definition_load_json(
                ids_json
                    .as_ref()
                    .map_or(std::ptr::null(), |json| json.as_ptr()),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payloads: Vec<GameActivityDefinitionPayload> =
                private::parse_json_ptr(out_json, "game activity definitions")?;
            Ok(payloads.into_iter().map(Self::from_payload).collect())
        }
    }

    /// Loads every activity definition available to the current game.
    pub fn load_all() -> Result<Vec<Self>, GameKitError> {
        Self::load(&[])
    }

    /// Loads achievement descriptions deep-linked from this activity definition.
    pub fn load_achievement_descriptions(
        &self,
    ) -> Result<Vec<AchievementDescription>, GameKitError> {
        let identifier =
            private::cstring_from_str(&self.identifier, "game activity definition identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_definition_load_achievement_descriptions_json(
                identifier.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payloads: Vec<crate::achievement::AchievementDescriptionPayload> =
                private::parse_json_ptr(out_json, "game activity achievement descriptions")?;
            Ok(payloads
                .into_iter()
                .map(AchievementDescription::from_payload)
                .collect())
        }
    }

    /// Loads leaderboards deep-linked from this activity definition.
    pub fn load_leaderboards(&self) -> Result<Vec<Leaderboard>, GameKitError> {
        let identifier =
            private::cstring_from_str(&self.identifier, "game activity definition identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_definition_load_leaderboards_json(
                identifier.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payloads: Vec<crate::leaderboard::LeaderboardPayload> =
                private::parse_json_ptr(out_json, "game activity leaderboards")?;
            Ok(payloads
                .into_iter()
                .map(Leaderboard::from_payload)
                .collect())
        }
    }

    /// Loads the definition image as TIFF bytes.
    pub fn load_image_tiff(&self) -> Result<Vec<u8>, GameKitError> {
        let identifier =
            private::cstring_from_str(&self.identifier, "game activity definition identifier")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_definition_load_image_tiff_json(
                identifier.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payload: BinaryPayload = private::parse_json_ptr(out_json, "game activity image")?;
            private::decode_base64(&payload.data_base64, "game activity image")
        }
    }

    pub(crate) fn from_payload(payload: GameActivityDefinitionPayload) -> Self {
        Self {
            identifier: payload.identifier,
            group_identifier: payload.group_identifier,
            title: payload.title,
            details: payload.details,
            default_properties: payload.default_properties,
            fallback_url: payload.fallback_url,
            supports_party_code: payload.supports_party_code,
            max_players: payload.max_players,
            min_players: payload.min_players,
            supports_unlimited_players: payload.supports_unlimited_players,
            play_style: GameActivityPlayStyle::from_raw(payload.play_style),
            release_state: payload.release_state,
        }
    }
}

impl GameActivity {
    pub(crate) const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns whether Game Center has a pending activity for the current game.
    pub fn has_pending_game_activities() -> Result<bool, GameKitError> {
        unsafe {
            let mut out_pending = false;
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_has_pending(&mut out_pending, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(out_pending)
        }
    }

    /// Returns the valid alphabet for party-code letters.
    pub fn valid_party_code_alphabet() -> Result<Vec<String>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_game_activity_valid_party_code_alphabet_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "game activity party-code alphabet")
        }
    }

    /// Validates a party-code string using `GameKit`'s built-in rules.
    pub fn is_valid_party_code(party_code: &str) -> Result<bool, GameKitError> {
        let party_code = private::cstring_from_str(party_code, "game activity party code")?;

        unsafe {
            let mut out_valid = false;
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_is_valid_party_code(
                party_code.as_ptr(),
                &mut out_valid,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(out_valid)
        }
    }

    /// Creates an unstarted activity from a definition.
    pub fn new(definition: &GameActivityDefinition) -> Result<Self, GameKitError> {
        let identifier = private::cstring_from_str(
            &definition.identifier,
            "game activity definition identifier",
        )?;

        unsafe {
            let mut out_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_game_activity_create(identifier.as_ptr(), &mut out_ptr, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(Self::from_raw(out_ptr))
        }
    }

    /// Creates and starts a new activity from a definition.
    pub fn start(definition: &GameActivityDefinition) -> Result<Self, GameKitError> {
        let identifier = private::cstring_from_str(
            &definition.identifier,
            "game activity definition identifier",
        )?;

        unsafe {
            let mut out_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_game_activity_start(identifier.as_ptr(), &mut out_ptr, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(Self::from_raw(out_ptr))
        }
    }

    /// Creates and starts a new activity with a custom party code.
    pub fn start_with_party_code(
        definition: &GameActivityDefinition,
        party_code: &str,
    ) -> Result<Self, GameKitError> {
        let identifier = private::cstring_from_str(
            &definition.identifier,
            "game activity definition identifier",
        )?;
        let party_code = private::cstring_from_str(party_code, "game activity party code")?;

        unsafe {
            let mut out_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_start_with_party_code(
                identifier.as_ptr(),
                party_code.as_ptr(),
                &mut out_ptr,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(Self::from_raw(out_ptr))
        }
    }

    /// Loads the latest snapshot for this activity.
    pub fn snapshot(&self) -> Result<GameActivitySnapshot, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_game_activity_snapshot_json(self.ptr, &mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payload: GameActivitySnapshotPayload =
                private::parse_json_ptr(out_json, "game activity")?;
            Ok(GameActivitySnapshot::from_payload(payload))
        }
    }

    /// Replaces the custom property dictionary for this activity.
    pub fn set_properties(
        &self,
        properties: &BTreeMap<String, String>,
    ) -> Result<(), GameKitError> {
        let properties_json = private::json_cstring(properties, "game activity properties")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_set_properties_json(
                self.ptr,
                properties_json.as_ptr(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Starts an activity that was previously created with [`GameActivity::new`].
    pub fn begin(&self) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_begin(self.ptr, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Pauses the activity.
    pub fn pause(&self) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_pause(self.ptr, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Resumes the activity.
    pub fn resume(&self) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_resume(self.ptr, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Ends the activity and commits associated scores and achievements.
    pub fn end(&self) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_end(self.ptr, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Builds a matchmaking request derived from the activity metadata, when possible.
    pub fn make_match_request(&self) -> Result<Option<MatchRequest>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_make_match_request_json(
                self.ptr,
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            let payload: Option<MatchRequestPayload> =
                private::parse_json_ptr(out_json, "game activity match request")?;
            Ok(payload.map(|payload| MatchRequest {
                min_players: payload.min_players,
                max_players: payload.max_players,
                player_group: payload.player_group,
                player_attributes: payload.player_attributes,
                recipient_ids: payload.recipient_ids,
                invite_message: payload.invite_message,
                default_number_of_players: Some(payload.default_number_of_players),
            }))
        }
    }

    /// Finds a peer-to-peer match derived from this activity.
    pub fn find_match(&self) -> Result<Match, GameKitError> {
        unsafe {
            let mut out_match_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_game_activity_find_match(self.ptr, &mut out_match_ptr, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(Match::from_raw(out_match_ptr))
        }
    }

    /// Finds hosted players derived from this activity.
    pub fn find_hosted_players(&self) -> Result<Vec<Player>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_find_hosted_players_json(
                self.ptr,
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "game activity hosted players")
        }
    }

    /// Associates a leaderboard score with this activity.
    pub fn set_score(&self, score: &Score) -> Result<(), GameKitError> {
        let payload = GameActivityScoreInputPayload {
            leaderboard_id: &score.leaderboard_id,
            value: score.value,
            context: score.context,
            player_game_id: score.player_game_id.as_deref(),
        };
        let json = private::json_cstring(&payload, "game activity score")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_game_activity_set_score_json(self.ptr, json.as_ptr(), &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Removes associated scores for the supplied leaderboards.
    pub fn remove_scores(&self, leaderboard_ids: &[&str]) -> Result<(), GameKitError> {
        let ids_json =
            private::json_cstring(leaderboard_ids, "game activity leaderboard identifiers")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_game_activity_remove_scores(self.ptr, ids_json.as_ptr(), &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Associates an achievement-progress update with this activity.
    pub fn set_progress(&self, achievement: &Achievement) -> Result<(), GameKitError> {
        let payload = GameActivityAchievementInputPayload {
            identifier: &achievement.identifier,
            percent_complete: achievement.percent_complete,
            shows_completion_banner: achievement.shows_completion_banner,
            player_game_id: achievement
                .player
                .as_ref()
                .map(|player| player.game_player_id.as_str()),
        };
        let json = private::json_cstring(&payload, "game activity achievement")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_game_activity_set_progress_json(self.ptr, json.as_ptr(), &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Marks an achievement complete for this activity.
    pub fn complete_achievement(&self, achievement: &Achievement) -> Result<(), GameKitError> {
        let payload = GameActivityAchievementInputPayload {
            identifier: &achievement.identifier,
            percent_complete: 100.0,
            shows_completion_banner: achievement.shows_completion_banner,
            player_game_id: achievement
                .player
                .as_ref()
                .map(|player| player.game_player_id.as_str()),
        };
        let json = private::json_cstring(&payload, "completed game activity achievement")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_complete_achievement_json(
                self.ptr,
                json.as_ptr(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Removes previously associated achievements from this activity.
    pub fn remove_achievements(&self, achievements: &[Achievement]) -> Result<(), GameKitError> {
        let payloads: Vec<_> = achievements
            .iter()
            .map(|achievement| GameActivityAchievementInputPayload {
                identifier: &achievement.identifier,
                percent_complete: achievement.percent_complete,
                shows_completion_banner: achievement.shows_completion_banner,
                player_game_id: achievement
                    .player
                    .as_ref()
                    .map(|player| player.game_player_id.as_str()),
            })
            .collect();
        let json = private::json_cstring(&payloads, "game activity achievements")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_game_activity_remove_achievements_json(
                self.ptr,
                json.as_ptr(),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }
}

impl Clone for GameActivity {
    fn clone(&self) -> Self {
        Self {
            ptr: unsafe { ffi::gk_game_activity_retain(self.ptr) },
        }
    }
}

impl Drop for GameActivity {
    fn drop(&mut self) {
        unsafe {
            ffi::gk_game_activity_release(self.ptr);
        }
    }
}

impl GameActivityPlayStyle {
    const fn from_raw(value: i32) -> Self {
        match value {
            1 => Self::Synchronous,
            2 => Self::Asynchronous,
            0 => Self::Unspecified,
            other => Self::Unknown(other),
        }
    }
}

impl GameActivityState {
    const fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::Initialized,
            1 => Self::Active,
            2 => Self::Paused,
            4 => Self::Ended,
            other => Self::Unknown(other),
        }
    }
}

impl GameActivitySnapshot {
    pub(crate) fn from_payload(payload: GameActivitySnapshotPayload) -> Self {
        Self {
            identifier: payload.identifier,
            activity_definition: GameActivityDefinition::from_payload(payload.activity_definition),
            properties: payload.properties,
            state: GameActivityState::from_raw(payload.state),
            party_code: payload.party_code,
            party_url: payload.party_url,
            creation_date: payload.creation_date,
            start_date: payload.start_date,
            last_resume_date: payload.last_resume_date,
            end_date: payload.end_date,
            duration_seconds: payload.duration_seconds,
            achievements: payload
                .achievements
                .into_iter()
                .map(|payload| {
                    Achievement::from_payload(crate::achievement::AchievementPayload {
                        identifier: payload.identifier,
                        percent_complete: payload.percent_complete,
                        is_completed: payload.is_completed,
                        last_reported_date: payload.last_reported_date,
                        shows_completion_banner: payload.shows_completion_banner,
                        player: payload.player,
                    })
                })
                .collect(),
            leaderboard_scores: payload
                .leaderboard_scores
                .into_iter()
                .map(|payload| Score {
                    leaderboard_id: payload.leaderboard_id,
                    value: payload.value,
                    context: payload.context,
                    player_game_id: payload.player_game_id,
                })
                .collect(),
        }
    }
}
