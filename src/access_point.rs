use core::ffi::c_char;

use serde::Deserialize;

use crate::{ffi, private, GameKitError};

/// Placement of the Game Center access point widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPointLocation {
    TopLeading,
    TopTrailing,
    BottomLeading,
    BottomTrailing,
}

/// Game Center states that can be shown when triggering the access point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPointState {
    Default,
    Leaderboards,
    Achievements,
    Challenges,
    LocalPlayerProfile,
    Dashboard,
    LocalPlayerFriendsList,
}

/// Snapshot of the current access point state.
#[derive(Debug, Clone, PartialEq)]
pub struct AccessPointSnapshot {
    pub is_active: bool,
    pub is_visible: bool,
    pub is_presenting_game_center: bool,
    pub location: AccessPointLocation,
    pub frame: AccessPointFrame,
}

/// Screen-space frame of the access point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccessPointFrame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AccessPoint;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccessPointPayload {
    is_active: bool,
    is_visible: bool,
    is_presenting_game_center: bool,
    location: String,
    frame: AccessPointFramePayload,
}

#[derive(Debug, Deserialize)]
struct AccessPointFramePayload {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl AccessPoint {
    /// Returns a zero-sized handle for the shared access point.
    #[must_use]
    pub const fn shared() -> Self {
        Self
    }

    /// Reads the current access point state.
    pub fn snapshot(self) -> Result<AccessPointSnapshot, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_access_point_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payload: AccessPointPayload = private::parse_json_ptr(out_json, "access point")?;
            Ok(AccessPointSnapshot {
                is_active: payload.is_active,
                is_visible: payload.is_visible,
                is_presenting_game_center: payload.is_presenting_game_center,
                location: location_from_str(&payload.location),
                frame: AccessPointFrame {
                    x: payload.frame.x,
                    y: payload.frame.y,
                    width: payload.frame.width,
                    height: payload.frame.height,
                },
            })
        }
    }

    /// Activates or deactivates the access point widget.
    pub fn set_active(self, active: bool) {
        unsafe {
            ffi::gk_access_point_set_active(active);
        }
    }

    /// Updates the widget location.
    pub fn set_location(self, location: AccessPointLocation) {
        unsafe {
            ffi::gk_access_point_set_location(location_to_i32(location));
        }
    }

    /// Triggers the access point using its default behavior.
    pub fn trigger(self) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_access_point_trigger(&mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Triggers the access point for a specific Game Center state.
    pub fn trigger_state(self, state: AccessPointState) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_access_point_trigger_state(state_to_i32(state), &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }
}

fn location_from_str(location: &str) -> AccessPointLocation {
    match location {
        "topTrailing" => AccessPointLocation::TopTrailing,
        "bottomLeading" => AccessPointLocation::BottomLeading,
        "bottomTrailing" => AccessPointLocation::BottomTrailing,
        _ => AccessPointLocation::TopLeading,
    }
}

const fn location_to_i32(location: AccessPointLocation) -> i32 {
    match location {
        AccessPointLocation::TopLeading => 0,
        AccessPointLocation::TopTrailing => 1,
        AccessPointLocation::BottomLeading => 2,
        AccessPointLocation::BottomTrailing => 3,
    }
}

const fn state_to_i32(state: AccessPointState) -> i32 {
    match state {
        AccessPointState::Default => -1,
        AccessPointState::Leaderboards => 0,
        AccessPointState::Achievements => 1,
        AccessPointState::Challenges => 2,
        AccessPointState::LocalPlayerProfile => 3,
        AccessPointState::Dashboard => 4,
        AccessPointState::LocalPlayerFriendsList => 5,
    }
}
