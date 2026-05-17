use core::ffi::{c_char, c_void};
use std::ffi::CStr;

use serde::Deserialize;

use doom_fish_utils::panic_safe::catch_user_panic;

use crate::{ffi, private, GameKitError, Player};

/// Mode for sending data through a match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendDataMode {
    Reliable,
    Unreliable,
}

/// Connection state of a player in a match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Unknown,
    Connected,
    Disconnected,
}

/// Framework error data for match failures.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameKitFrameworkErrorData {
    #[allow(dead_code)]
    kind: String,
    pub domain: String,
    pub code: i64,
    pub localized_description: String,
}

/// Events received from a match delegate.
#[derive(Debug, Clone)]
pub enum MatchEvent {
    ReceivedData {
        player: Player,
        data: Vec<u8>,
    },
    ConnectionStateChanged {
        player: Player,
        state: ConnectionState,
    },
    Failed {
        error: Option<GameKitFrameworkErrorData>,
    },
}

/// Represents a Game Center real-time match.
#[derive(Debug)]
pub struct Match {
    pub(crate) ptr: *mut c_void,
}

/// SAFETY: `GKMatch` is an Objective-C object whose documented thread-safety contract
/// allows concurrent reads and sends from any thread. The `ptr` field is never
/// aliased mutably outside of the Obj-C runtime.
unsafe impl Send for Match {}

impl Match {
    pub(crate) const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns every player currently associated with the match.
    pub fn players(&self) -> Result<Vec<Player>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_match_players_json(self.ptr, &mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "match players")
        }
    }

    /// Returns the connected players for the match.
    pub fn connected_players(&self) -> Result<Vec<Player>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_match_connected_players_json(self.ptr, &mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "connected players")
        }
    }

    /// Returns the number of players that are still expected to join.
    #[must_use]
    pub fn expected_player_count(&self) -> usize {
        unsafe { ffi::gk_match_expected_player_count(self.ptr) }
    }

    /// Sends data to the supplied player identifiers.
    pub fn send_data(
        &self,
        data: &[u8],
        player_ids: &[&str],
        mode: SendDataMode,
    ) -> Result<(), GameKitError> {
        let ids_json = private::json_cstring(player_ids, "player identifiers")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_match_send_data(
                self.ptr,
                data.as_ptr(),
                data.len(),
                ids_json.as_ptr(),
                send_mode_to_i32(mode),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Sends data to every connected player.
    pub fn send_data_to_all(&self, data: &[u8], mode: SendDataMode) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_match_send_data_to_all(
                self.ptr,
                data.as_ptr(),
                data.len(),
                send_mode_to_i32(mode),
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Chooses the best host among currently-connected players.
    pub fn choose_best_hosting_player(&self) -> Result<Option<Player>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_match_choose_best_hosting_player_json(
                self.ptr,
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "best hosting player")
        }
    }

    /// Requests a rematch with the same set of players.
    pub fn rematch(&self) -> Result<Self, GameKitError> {
        unsafe {
            let mut out_match_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_match_rematch(self.ptr, &mut out_match_ptr, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(Self::from_raw(out_match_ptr))
        }
    }

    /// Sets a delegate to receive match events.
    pub fn set_delegate<F: Fn(MatchEvent) + Send + 'static>(&self, handler: F) -> MatchDelegate {
        let boxed: Box<dyn Fn(MatchEvent) + Send + 'static> = Box::new(handler);
        let handler_ptr: *mut Box<dyn Fn(MatchEvent) + Send + 'static> =
            Box::into_raw(Box::new(boxed));

        unsafe {
            ffi::gk_match_set_callbacks(
                self.ptr,
                Some(match_data_trampoline),
                Some(match_state_trampoline),
                Some(match_failure_trampoline),
                handler_ptr.cast(),
            );
        }

        MatchDelegate {
            match_ptr: self.ptr,
            handler_ptr,
        }
    }

    /// Disconnects from the active match.
    pub fn disconnect(&self) {
        unsafe {
            ffi::gk_match_disconnect(self.ptr);
        }
    }
}

impl Clone for Match {
    fn clone(&self) -> Self {
        Self {
            ptr: unsafe { ffi::gk_match_retain(self.ptr) },
        }
    }
}

impl Drop for Match {
    fn drop(&mut self) {
        unsafe {
            ffi::gk_match_release(self.ptr);
        }
    }
}

/// A guard that manages match event callbacks. Clears callbacks when dropped.
pub struct MatchDelegate {
    match_ptr: *mut c_void,
    handler_ptr: *mut Box<dyn Fn(MatchEvent) + Send + 'static>,
}

/// SAFETY: `MatchDelegate` owns a raw handler pointer that is never accessed
/// from multiple threads simultaneously; the match delegate callbacks are
/// serialised by the `GameKit` runtime.
unsafe impl Send for MatchDelegate {}

impl Drop for MatchDelegate {
    fn drop(&mut self) {
        unsafe {
            ffi::gk_match_clear_callbacks(self.match_ptr);
            drop(Box::from_raw(self.handler_ptr));
        }
    }
}

unsafe extern "C" fn match_data_trampoline(
    refcon: *mut c_void,
    player_json: *const c_char,
    data: *const u8,
    len: usize,
) {
    // SAFETY: refcon is a valid `Box<Box<dyn Fn(MatchEvent) + Send>>` allocated in
    // `Match::set_delegate` and kept alive by `MatchDelegate`.
    // `data` is a valid buffer of `len` bytes for the duration of this callback.
    let handler = &*(refcon.cast::<Box<dyn Fn(MatchEvent) + Send + 'static>>());
    if let Ok(player_str) = CStr::from_ptr(player_json).to_str() {
        if let Ok(player) = serde_json::from_str::<Player>(player_str) {
            let data_vec = std::slice::from_raw_parts(data, len).to_vec();
            catch_user_panic("match_data_trampoline", || {
                handler(MatchEvent::ReceivedData {
                    player,
                    data: data_vec,
                });
            });
        }
    }
}

unsafe extern "C" fn match_state_trampoline(
    refcon: *mut c_void,
    player_json: *const c_char,
    state: i32,
) {
    // SAFETY: refcon is a valid `Box<Box<dyn Fn(MatchEvent) + Send>>` allocated in
    // `Match::set_delegate` and kept alive by `MatchDelegate`.
    let handler = &*(refcon.cast::<Box<dyn Fn(MatchEvent) + Send + 'static>>());
    if let Ok(player_str) = CStr::from_ptr(player_json).to_str() {
        if let Ok(player) = serde_json::from_str::<Player>(player_str) {
            catch_user_panic("match_state_trampoline", || {
                handler(MatchEvent::ConnectionStateChanged {
                    player,
                    state: match state {
                        1 => ConnectionState::Connected,
                        2 => ConnectionState::Disconnected,
                        _ => ConnectionState::Unknown,
                    },
                });
            });
        }
    }
}

unsafe extern "C" fn match_failure_trampoline(refcon: *mut c_void, error_json: *const c_char) {
    // SAFETY: refcon is a valid `Box<Box<dyn Fn(MatchEvent) + Send>>` allocated in
    // `Match::set_delegate` and kept alive by `MatchDelegate`.
    let handler = &*(refcon.cast::<Box<dyn Fn(MatchEvent) + Send + 'static>>());
    let error = if error_json.is_null() {
        None
    } else if let Ok(error_str) = CStr::from_ptr(error_json).to_str() {
        serde_json::from_str::<GameKitFrameworkErrorData>(error_str).ok()
    } else {
        None
    };
    catch_user_panic("match_failure_trampoline", || {
        handler(MatchEvent::Failed { error });
    });
}

const fn send_mode_to_i32(mode: SendDataMode) -> i32 {
    match mode {
        SendDataMode::Reliable => 0,
        SendDataMode::Unreliable => 1,
    }
}
