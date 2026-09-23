use core::ffi::{c_char, c_void};

use serde::Deserialize;

use doom_fish_utils::panic_safe::catch_user_panic;

use crate::leaderboard_entry::{player_scope_to_i32, time_scope_to_i32};
use crate::{ffi, private, GameActivity, GameKitError, Player, PlayerScope, TimeScope};

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

/// Game Center content the access point can open.
#[derive(Clone, Copy)]
pub enum AccessPointTrigger<'a> {
    Default,
    State(AccessPointState),
    Achievement(&'a str),
    LeaderboardSet(&'a str),
    Leaderboard {
        id: &'a str,
        player_scope: PlayerScope,
        time_scope: TimeScope,
    },
    Player(&'a Player),
    PlayTogether,
    Challenges,
    ChallengeDefinition(&'a str),
    GameActivityDefinition(&'a str),
    GameActivity(&'a GameActivity),
    Friending,
    Arcade,
}

type AccessPointCompletion = Box<dyn FnOnce(Result<(), GameKitError>) + Send + 'static>;
type CompletionFn = unsafe extern "C" fn(*mut c_void, i32, *mut c_char);
type DiscardFn = unsafe extern "C" fn(*mut c_void);

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

    /// Opens Game Center with the access point's default behavior and returns without waiting.
    pub fn trigger(self) -> Result<(), GameKitError> {
        send_trigger(AccessPointTrigger::Default, None)
    }

    /// Opens Game Center at `state` and returns without waiting.
    pub fn trigger_state(self, state: AccessPointState) -> Result<(), GameKitError> {
        send_trigger(AccessPointTrigger::State(state), None)
    }

    /// Opens `target` and returns without waiting.
    ///
    /// `completion` runs on the main thread with `Ok(())` once Game Center closes, or with the
    /// error that prevented it from opening after this call returned. It is dropped without being
    /// called when this call returns an error or when `GameKit` never reports completion.
    pub fn trigger_target<F>(
        self,
        target: AccessPointTrigger<'_>,
        completion: F,
    ) -> Result<(), GameKitError>
    where
        F: FnOnce(Result<(), GameKitError>) + Send + 'static,
    {
        send_trigger(target, Some(Box::new(completion)))
    }
}

fn send_trigger(
    target: AccessPointTrigger<'_>,
    completion: Option<AccessPointCompletion>,
) -> Result<(), GameKitError> {
    let mut state = 0;
    let mut identifier = None;
    let mut player_scope = 0;
    let mut time_scope = 0;
    let mut activity: *mut c_void = std::ptr::null_mut();
    let kind = match target {
        AccessPointTrigger::Default => 0,
        AccessPointTrigger::State(value) => {
            state = state_to_i32(value);
            1
        }
        AccessPointTrigger::Achievement(id) => {
            identifier = Some(id);
            2
        }
        AccessPointTrigger::LeaderboardSet(id) => {
            identifier = Some(id);
            3
        }
        AccessPointTrigger::Leaderboard {
            id,
            player_scope: players,
            time_scope: period,
        } => {
            identifier = Some(id);
            player_scope = player_scope_to_i32(players);
            time_scope = time_scope_to_i32(period);
            4
        }
        AccessPointTrigger::Player(player) => {
            identifier = Some(player.game_player_id.as_str());
            5
        }
        AccessPointTrigger::PlayTogether => 6,
        AccessPointTrigger::Challenges => 7,
        AccessPointTrigger::ChallengeDefinition(id) => {
            identifier = Some(id);
            8
        }
        AccessPointTrigger::GameActivityDefinition(id) => {
            identifier = Some(id);
            9
        }
        AccessPointTrigger::GameActivity(game_activity) => {
            activity = game_activity.ptr;
            10
        }
        AccessPointTrigger::Friending => 11,
        AccessPointTrigger::Arcade => 12,
    };
    let identifier = identifier
        .map(|value| private::cstring_from_str(value, "access point identifier"))
        .transpose()?;
    let (callback, refcon, discard): (Option<CompletionFn>, *mut c_void, Option<DiscardFn>) =
        completion.map_or((None, std::ptr::null_mut(), None), |completion| {
            (
                Some(access_point_completion_trampoline),
                Box::into_raw(Box::new(completion)).cast(),
                Some(access_point_completion_discard),
            )
        });

    unsafe {
        let mut out_error: *mut c_char = std::ptr::null_mut();
        let status = ffi::gk_access_point_trigger_target(
            kind,
            state,
            identifier
                .as_ref()
                .map_or(std::ptr::null(), |value| value.as_ptr()),
            player_scope,
            time_scope,
            activity,
            callback,
            refcon,
            discard,
            &raw mut out_error,
        );
        if status != ffi::status::OK {
            return Err(private::error_from_status(status, out_error));
        }
        Ok(())
    }
}

unsafe extern "C" fn access_point_completion_trampoline(
    refcon: *mut c_void,
    status: i32,
    error: *mut c_char,
) {
    let completion = Box::from_raw(refcon.cast::<AccessPointCompletion>());
    let result = if status == ffi::status::OK {
        Ok(())
    } else {
        Err(private::error_from_status(status, error))
    };
    catch_user_panic("access_point_completion_trampoline", move || {
        completion(result);
    });
}

unsafe extern "C" fn access_point_completion_discard(refcon: *mut c_void) {
    let completion = Box::from_raw(refcon.cast::<AccessPointCompletion>());
    catch_user_panic("access_point_completion_discard", move || drop(completion));
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
