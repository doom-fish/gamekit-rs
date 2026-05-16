use core::ffi::c_char;

use serde::Serialize;

use crate::{ffi, private, GameKitError};

/// Represents a Game Center achievement.
#[derive(Debug, Clone)]
pub struct Achievement {
    pub identifier: String,
    pub percent_complete: f64,
    pub is_completed: bool,
    pub last_reported_date: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AchievementInputPayload {
    identifier: String,
    percent_complete: f64,
}

impl Achievement {
    /// Reports achievements to Game Center.
    ///
    /// Each item is a tuple of (`identifier`, `percent_complete`).
    pub fn report(items: &[(&str, f64)]) -> Result<(), GameKitError> {
        let payloads: Vec<AchievementInputPayload> = items
            .iter()
            .map(|(id, pct)| AchievementInputPayload {
                identifier: (*id).to_owned(),
                percent_complete: *pct,
            })
            .collect();

        let json = private::json_cstring(&payloads, "achievements")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_achievement_report_json(json.as_ptr(), &mut out_error);

            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            Ok(())
        }
    }

    /// Reports a slice of Achievement instances.
    pub fn report_all(achievements: &[Self]) -> Result<(), GameKitError> {
        let payloads: Vec<AchievementInputPayload> = achievements
            .iter()
            .map(|a| AchievementInputPayload {
                identifier: a.identifier.clone(),
                percent_complete: a.percent_complete,
            })
            .collect();

        let json = private::json_cstring(&payloads, "achievements")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_achievement_report_json(json.as_ptr(), &mut out_error);

            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            Ok(())
        }
    }
}
