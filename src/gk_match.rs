use core::ffi::{c_char, c_void};
use std::ffi::CStr;

use serde::Deserialize;

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
    ReceivedData { player: Player, data: Vec<u8> },
    ConnectionStateChanged { player: Player, state: ConnectionState },
    Failed { error: Option<GameKitFrameworkErrorData> },
}

/// Represents a Game Center match.
pub struct Match {
    ptr: *mut c_void,
}

unsafe impl Send for Match {}

impl Match {
    pub(crate) const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns the list of connected players.
    pub fn connected_players(&self) -> Result<Vec<Player>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_match_connected_players_json(
                self.ptr,
                &mut out_json,
                &mut out_error,
            );

            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            private::parse_json_ptr(out_json, "connected players")
        }
    }

    /// Sends data to specific players.
    pub fn send_data(
        &self,
        data: &[u8],
        player_ids: &[&str],
        mode: SendDataMode,
    ) -> Result<(), GameKitError> {
        let ids_json = private::json_cstring(&player_ids, "player IDs")?;
        let mode_i32 = match mode {
            SendDataMode::Reliable => 0,
            SendDataMode::Unreliable => 1,
        };

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_match_send_data(
                self.ptr,
                data.as_ptr(),
                data.len(),
                ids_json.as_ptr(),
                mode_i32,
                &mut out_error,
            );

            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            Ok(())
        }
    }

    /// Sends data to all connected players.
    pub fn send_data_to_all(&self, data: &[u8], mode: SendDataMode) -> Result<(), GameKitError> {
        let mode_i32 = match mode {
            SendDataMode::Reliable => 0,
            SendDataMode::Unreliable => 1,
        };

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_match_send_data_to_all(
                self.ptr,
                data.as_ptr(),
                data.len(),
                mode_i32,
                &mut out_error,
            );

            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }

            Ok(())
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

    /// Disconnects from the match.
    pub fn disconnect(&self) {
        unsafe {
            ffi::gk_match_disconnect(self.ptr);
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
    let handler = &*(refcon.cast::<Box<dyn Fn(MatchEvent) + Send + 'static>>());
    if let Ok(player_str) = CStr::from_ptr(player_json).to_str() {
        if let Ok(player) = serde_json::from_str::<Player>(player_str) {
            let data_vec = std::slice::from_raw_parts(data, len).to_vec();
            handler(MatchEvent::ReceivedData {
                player,
                data: data_vec,
            });
        }
    }
}

unsafe extern "C" fn match_state_trampoline(
    refcon: *mut c_void,
    player_json: *const c_char,
    state: i32,
) {
    let handler = &*(refcon.cast::<Box<dyn Fn(MatchEvent) + Send + 'static>>());
    if let Ok(player_str) = CStr::from_ptr(player_json).to_str() {
        if let Ok(player) = serde_json::from_str::<Player>(player_str) {
            let connection_state = match state {
                1 => ConnectionState::Connected,
                2 => ConnectionState::Disconnected,
                _ => ConnectionState::Unknown,
            };
            handler(MatchEvent::ConnectionStateChanged {
                player,
                state: connection_state,
            });
        }
    }
}

unsafe extern "C" fn match_failure_trampoline(refcon: *mut c_void, error_json: *const c_char) {
    let handler = &*(refcon.cast::<Box<dyn Fn(MatchEvent) + Send + 'static>>());
    let error = if error_json.is_null() {
        None
    } else if let Ok(error_str) = CStr::from_ptr(error_json).to_str() {
        serde_json::from_str::<GameKitFrameworkErrorData>(error_str).ok()
    } else {
        None
    };
    handler(MatchEvent::Failed { error });
}
