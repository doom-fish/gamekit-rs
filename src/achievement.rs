use core::ffi::c_char;

use serde::Deserialize;

use crate::{ffi, private, GameKitError, Player};

/// Represents a Game Center achievement.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq)]
pub struct Achievement {
    pub identifier: String,
    pub percent_complete: f64,
    pub is_completed: bool,
    pub last_reported_date: Option<String>,
    pub shows_completion_banner: bool,
    pub player: Option<Player>,
}

/// Describes an achievement defined in App Store Connect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementDescription {
    pub identifier: Option<String>,
    pub group_identifier: Option<String>,
    pub title: Option<String>,
    pub achieved_description: Option<String>,
    pub unachieved_description: Option<String>,
    pub maximum_points: i64,
    pub is_hidden: bool,
    pub is_replayable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AchievementPayload {
    identifier: String,
    percent_complete: f64,
    is_completed: bool,
    last_reported_date: Option<String>,
    shows_completion_banner: bool,
    player: Option<Player>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AchievementDescriptionPayload {
    identifier: Option<String>,
    group_identifier: Option<String>,
    title: Option<String>,
    achieved_description: Option<String>,
    unachieved_description: Option<String>,
    maximum_points: i64,
    is_hidden: bool,
    is_replayable: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AchievementInputPayload<'a> {
    identifier: &'a str,
    percent_complete: f64,
    shows_completion_banner: bool,
    player_game_id: Option<&'a str>,
}

impl Achievement {
    /// Loads all achievements for the local player.
    pub fn load() -> Result<Vec<Self>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_achievement_load_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payloads: Vec<AchievementPayload> =
                private::parse_json_ptr(out_json, "achievements")?;
            Ok(payloads.into_iter().map(Self::from_payload).collect())
        }
    }

    /// Loads achievement descriptions defined for the current game.
    pub fn load_descriptions() -> Result<Vec<AchievementDescription>, GameKitError> {
        AchievementDescription::load()
    }

    /// Reports achievements to Game Center.
    pub fn report(items: &[(&str, f64)]) -> Result<(), GameKitError> {
        let payloads: Vec<AchievementInputPayload<'_>> = items
            .iter()
            .map(|(identifier, percent_complete)| AchievementInputPayload {
                identifier,
                percent_complete: *percent_complete,
                shows_completion_banner: false,
                player_game_id: None,
            })
            .collect();
        report_payloads(&payloads)
    }

    /// Reports a slice of [`Achievement`] values.
    pub fn report_all(achievements: &[Self]) -> Result<(), GameKitError> {
        let payloads: Vec<AchievementInputPayload<'_>> = achievements
            .iter()
            .map(|achievement| AchievementInputPayload {
                identifier: &achievement.identifier,
                percent_complete: achievement.percent_complete,
                shows_completion_banner: achievement.shows_completion_banner,
                player_game_id: achievement.player.as_ref().map(|player| player.game_player_id.as_str()),
            })
            .collect();
        report_payloads(&payloads)
    }

    /// Resets all achievements for the local player.
    pub fn reset() -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_achievement_reset(&mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    fn from_payload(payload: AchievementPayload) -> Self {
        Self {
            identifier: payload.identifier,
            percent_complete: payload.percent_complete,
            is_completed: payload.is_completed,
            last_reported_date: payload.last_reported_date,
            shows_completion_banner: payload.shows_completion_banner,
            player: payload.player,
        }
    }
}

impl AchievementDescription {
    /// Loads all achievement descriptions for the local game.
    pub fn load() -> Result<Vec<Self>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_achievement_descriptions_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payloads: Vec<AchievementDescriptionPayload> =
                private::parse_json_ptr(out_json, "achievement descriptions")?;
            Ok(payloads.into_iter().map(Self::from_payload).collect())
        }
    }

    fn from_payload(payload: AchievementDescriptionPayload) -> Self {
        Self {
            identifier: payload.identifier,
            group_identifier: payload.group_identifier,
            title: payload.title,
            achieved_description: payload.achieved_description,
            unachieved_description: payload.unachieved_description,
            maximum_points: payload.maximum_points,
            is_hidden: payload.is_hidden,
            is_replayable: payload.is_replayable,
        }
    }
}

fn report_payloads(payloads: &[AchievementInputPayload<'_>]) -> Result<(), GameKitError> {
    let json = private::json_cstring(payloads, "achievements")?;

    unsafe {
        let mut out_error: *mut c_char = std::ptr::null_mut();
        let status = ffi::gk_achievement_report_json(json.as_ptr(), &mut out_error);
        if status != ffi::status::OK {
            return Err(private::error_from_status(status, out_error));
        }
        Ok(())
    }
}
