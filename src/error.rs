use core::ffi::c_char;
use std::fmt;

use serde::Deserialize;

use crate::ffi;
use crate::private::take_string;

/// The exported `GameKit` error-domain constant.
pub const ERROR_DOMAIN: &str = "GKErrorDomain";

/// Typed `GameKit` framework error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    Unknown,
    Cancelled,
    CommunicationsFailure,
    UserDenied,
    InvalidCredentials,
    NotAuthenticated,
    AuthenticationInProgress,
    InvalidPlayer,
    ScoreNotSet,
    ParentalControlsBlocked,
    PlayerStatusExceedsMaximumLength,
    PlayerStatusInvalid,
    MatchRequestInvalid,
    Underage,
    GameUnrecognized,
    NotSupported,
    InvalidParameter,
    UnexpectedConnection,
    ChallengeInvalid,
    TurnBasedMatchDataTooLarge,
    TurnBasedTooManySessions,
    TurnBasedInvalidParticipant,
    TurnBasedInvalidTurn,
    TurnBasedInvalidState,
    InvitationsDisabled,
    PlayerPhotoFailure,
    UbiquityContainerUnavailable,
    MatchNotConnected,
    GameSessionRequestInvalid,
    RestrictedToAutomatch,
    ApiNotAvailable,
    NotAuthorized,
    ConnectionTimeout,
    ApiObsolete,
    ICloudUnavailable,
    LockdownMode,
    AppUnlisted,
    DebugMode,
    FriendListDescriptionMissing,
    FriendListRestricted,
    FriendListDenied,
    FriendRequestNotAvailable,
    Other(i64),
}

impl ErrorCode {
    #[must_use]
    pub const fn from_raw(value: i64) -> Self {
        match value {
            1 => Self::Unknown,
            2 => Self::Cancelled,
            3 => Self::CommunicationsFailure,
            4 => Self::UserDenied,
            5 => Self::InvalidCredentials,
            6 => Self::NotAuthenticated,
            7 => Self::AuthenticationInProgress,
            8 => Self::InvalidPlayer,
            9 => Self::ScoreNotSet,
            10 => Self::ParentalControlsBlocked,
            11 => Self::PlayerStatusExceedsMaximumLength,
            12 => Self::PlayerStatusInvalid,
            13 => Self::MatchRequestInvalid,
            14 => Self::Underage,
            15 => Self::GameUnrecognized,
            16 => Self::NotSupported,
            17 => Self::InvalidParameter,
            18 => Self::UnexpectedConnection,
            19 => Self::ChallengeInvalid,
            20 => Self::TurnBasedMatchDataTooLarge,
            21 => Self::TurnBasedTooManySessions,
            22 => Self::TurnBasedInvalidParticipant,
            23 => Self::TurnBasedInvalidTurn,
            24 => Self::TurnBasedInvalidState,
            25 => Self::InvitationsDisabled,
            26 => Self::PlayerPhotoFailure,
            27 => Self::UbiquityContainerUnavailable,
            28 => Self::MatchNotConnected,
            29 => Self::GameSessionRequestInvalid,
            30 => Self::RestrictedToAutomatch,
            31 => Self::ApiNotAvailable,
            32 => Self::NotAuthorized,
            33 => Self::ConnectionTimeout,
            34 => Self::ApiObsolete,
            35 => Self::ICloudUnavailable,
            36 => Self::LockdownMode,
            37 => Self::AppUnlisted,
            38 => Self::DebugMode,
            100 => Self::FriendListDescriptionMissing,
            101 => Self::FriendListRestricted,
            102 => Self::FriendListDenied,
            103 => Self::FriendRequestNotAvailable,
            other => Self::Other(other),
        }
    }

    #[must_use]
    pub const fn raw_value(self) -> i64 {
        match self {
            Self::Unknown => 1,
            Self::Cancelled => 2,
            Self::CommunicationsFailure => 3,
            Self::UserDenied => 4,
            Self::InvalidCredentials => 5,
            Self::NotAuthenticated => 6,
            Self::AuthenticationInProgress => 7,
            Self::InvalidPlayer => 8,
            Self::ScoreNotSet => 9,
            Self::ParentalControlsBlocked => 10,
            Self::PlayerStatusExceedsMaximumLength => 11,
            Self::PlayerStatusInvalid => 12,
            Self::MatchRequestInvalid => 13,
            Self::Underage => 14,
            Self::GameUnrecognized => 15,
            Self::NotSupported => 16,
            Self::InvalidParameter => 17,
            Self::UnexpectedConnection => 18,
            Self::ChallengeInvalid => 19,
            Self::TurnBasedMatchDataTooLarge => 20,
            Self::TurnBasedTooManySessions => 21,
            Self::TurnBasedInvalidParticipant => 22,
            Self::TurnBasedInvalidTurn => 23,
            Self::TurnBasedInvalidState => 24,
            Self::InvitationsDisabled => 25,
            Self::PlayerPhotoFailure => 26,
            Self::UbiquityContainerUnavailable => 27,
            Self::MatchNotConnected => 28,
            Self::GameSessionRequestInvalid => 29,
            Self::RestrictedToAutomatch => 30,
            Self::ApiNotAvailable => 31,
            Self::NotAuthorized => 32,
            Self::ConnectionTimeout => 33,
            Self::ApiObsolete => 34,
            Self::ICloudUnavailable => 35,
            Self::LockdownMode => 36,
            Self::AppUnlisted => 37,
            Self::DebugMode => 38,
            Self::FriendListDescriptionMissing => 100,
            Self::FriendListRestricted => 101,
            Self::FriendListDenied => 102,
            Self::FriendRequestNotAvailable => 103,
            Self::Other(other) => other,
        }
    }
}

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

impl GameKitFrameworkError {
    /// Returns the typed `GameKit` error code when the framework error came from `GKErrorDomain`.
    #[must_use]
    pub fn error_code(&self) -> Option<ErrorCode> {
        (self.domain == ERROR_DOMAIN).then_some(ErrorCode::from_raw(self.code))
    }
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
