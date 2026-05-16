use core::ffi::c_char;
use std::fmt;

use serde::Deserialize;

use crate::ffi;
use crate::private::take_string;

#[derive(Debug, Clone)]
pub enum GameKitError {
    TimedOut(String),
    NotAuthenticated,
    NotFound(String),
    Unavailable(String),
    Framework(GameKitFrameworkError),
    Unknown(String),
}

impl fmt::Display for GameKitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TimedOut(message)
            | Self::NotFound(message)
            | Self::Unavailable(message)
            | Self::Unknown(message) => formatter.write_str(message),
            Self::NotAuthenticated => formatter.write_str("not authenticated"),
            Self::Framework(error) => write!(
                formatter,
                "{} (domain={}, code={})",
                error.localized_description, error.domain, error.code
            ),
        }
    }
}

impl std::error::Error for GameKitError {}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GameKitFrameworkError {
    pub domain: String,
    pub code: i64,
    pub localized_description: String,
}

#[derive(Debug, Deserialize)]
struct FrameworkErrorPayload {
    #[allow(dead_code)]
    kind: String,
    domain: String,
    code: i64,
    #[serde(rename = "localizedDescription")]
    localized_description: String,
}

pub(crate) unsafe fn from_swift(status: i32, err_msg: *mut c_char) -> GameKitError {
    let message = take_string(err_msg);
    match status {
        ffi::status::TIMED_OUT => GameKitError::TimedOut(
            message.unwrap_or_else(|| "GameKit operation timed out".to_owned()),
        ),
        ffi::status::NOT_AUTHENTICATED => GameKitError::NotAuthenticated,
        ffi::status::NOT_FOUND => GameKitError::NotFound(
            message.unwrap_or_else(|| "GameKit resource not found".to_owned()),
        ),
        ffi::status::UNAVAILABLE => GameKitError::Unavailable(
            message.unwrap_or_else(|| "GameKit API is unavailable on this SDK or OS".to_owned()),
        ),
        ffi::status::FRAMEWORK_ERROR => parse_framework_error(message),
        _ => GameKitError::Unknown(
            message
                .unwrap_or_else(|| format!("GameKit bridge returned unexpected status {status}")),
        ),
    }
}

fn parse_framework_error(message: Option<String>) -> GameKitError {
    message.map_or_else(
        || {
            GameKitError::Framework(GameKitFrameworkError {
                domain: "GameKit".to_owned(),
                code: i64::from(ffi::status::FRAMEWORK_ERROR),
                localized_description: "GameKit framework error".to_owned(),
            })
        },
        |json| {
            serde_json::from_str::<FrameworkErrorPayload>(&json).map_or_else(
                |_| {
                    GameKitError::Framework(GameKitFrameworkError {
                        domain: "GameKit".to_owned(),
                        code: i64::from(ffi::status::FRAMEWORK_ERROR),
                        localized_description: json,
                    })
                },
                |payload| {
                    GameKitError::Framework(GameKitFrameworkError {
                        domain: payload.domain,
                        code: payload.code,
                        localized_description: payload.localized_description,
                    })
                },
            )
        },
    )
}
