use core::ffi::{c_char, c_void};
use std::ffi::CStr;

use serde::Deserialize;

use crate::{ffi, private, GameKitError, Player};

/// Represents the local authenticated player.
#[derive(Debug, Clone)]
#[allow(clippy::unsafe_derive_deserialize)]
pub struct LocalPlayer {
    pub is_authenticated: bool,
    pub is_underage: bool,
    pub is_multiplayer_gaming_restricted: bool,
    pub player: Player,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalPlayerPayload {
    is_authenticated: bool,
    is_underage: bool,
    is_multiplayer_gaming_restricted: bool,
    player: Player,
}

impl LocalPlayer {
    /// Returns a snapshot of the local player's authentication state.
    pub fn local() -> Result<Self, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_local_player_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payload: LocalPlayerPayload = private::parse_json_ptr(out_json, "LocalPlayer")?;
            Ok(Self {
                is_authenticated: payload.is_authenticated,
                is_underage: payload.is_underage,
                is_multiplayer_gaming_restricted: payload.is_multiplayer_gaming_restricted,
                player: payload.player,
            })
        }
    }
}

/// Authentication event received by the auth observer.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthEvent {
    pub has_view_controller: bool,
    pub error: Option<AuthEventError>,
}

/// Error information from an authentication event.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthEventError {
    pub domain: String,
    pub code: i64,
    pub localized_description: String,
}

/// A guard that registers an authentication handler. The handler is cleared when dropped.
pub struct AuthObserver {
    handler_ptr: *mut Box<dyn Fn(AuthEvent) + Send + 'static>,
}

unsafe impl Send for AuthObserver {}

impl AuthObserver {
    /// Creates a new authentication observer with the given handler.
    pub fn new<F: Fn(AuthEvent) + Send + 'static>(handler: F) -> Self {
        let boxed: Box<dyn Fn(AuthEvent) + Send + 'static> = Box::new(handler);
        let handler_ptr: *mut Box<dyn Fn(AuthEvent) + Send + 'static> =
            Box::into_raw(Box::new(boxed));
        unsafe {
            ffi::gk_authenticate_handler_set(Some(auth_trampoline), handler_ptr.cast());
        }
        Self { handler_ptr }
    }
}

impl Drop for AuthObserver {
    fn drop(&mut self) {
        unsafe {
            ffi::gk_authenticate_handler_clear();
            drop(Box::from_raw(self.handler_ptr));
        }
    }
}

unsafe extern "C" fn auth_trampoline(refcon: *mut c_void, event_json: *const c_char) {
    let handler = &*(refcon.cast::<Box<dyn Fn(AuthEvent) + Send + 'static>>());
    if let Ok(event_str) = CStr::from_ptr(event_json).to_str() {
        if let Ok(event) = serde_json::from_str::<AuthEvent>(event_str) {
            handler(event);
        }
    }
}
