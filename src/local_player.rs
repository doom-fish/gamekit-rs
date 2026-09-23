use core::ffi::{c_char, c_void};
use core::ptr::NonNull;
use std::ffi::CStr;
use std::sync::{Mutex, PoisonError};

use serde::Deserialize;

use doom_fish_utils::callback_context::CallbackContext;

use crate::{ffi, private, GameKitError, Player};

/// The exported authentication-change notification name.
pub const PLAYER_AUTHENTICATION_DID_CHANGE_NOTIFICATION_NAME: &str =
    "GKPlayerAuthenticationDidChangeNotificationName";

/// Represents the local authenticated player.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct LocalPlayer {
    pub is_authenticated: bool,
    pub is_underage: bool,
    pub is_multiplayer_gaming_restricted: bool,
    pub is_personalized_communication_restricted: bool,
    pub is_presenting_friend_request_view_controller: bool,
    pub player: Player,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalPlayerPayload {
    is_authenticated: bool,
    is_underage: bool,
    is_multiplayer_gaming_restricted: bool,
    is_personalized_communication_restricted: bool,
    is_presenting_friend_request_view_controller: bool,
    player: Player,
}

/// Result of identity verification signature generation.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IdentityVerificationSignature {
    pub public_key_url: String,
    pub signature_base64: String,
    pub salt_base64: String,
    pub timestamp: u64,
}

/// Authorization state for friends list access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FriendsAuthorizationStatus {
    NotDetermined,
    Restricted,
    Denied,
    Authorized,
}

impl LocalPlayer {
    /// Returns a snapshot of the local player's authentication state.
    pub fn local() -> Result<Self, GameKitError> {
        let payload: LocalPlayerPayload = unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_local_player_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "local player")?
        };

        Ok(Self {
            is_authenticated: payload.is_authenticated,
            is_underage: payload.is_underage,
            is_multiplayer_gaming_restricted: payload.is_multiplayer_gaming_restricted,
            is_personalized_communication_restricted: payload
                .is_personalized_communication_restricted,
            is_presenting_friend_request_view_controller: payload
                .is_presenting_friend_request_view_controller,
            player: payload.player,
        })
    }

    /// Loads the local player's recent players.
    pub fn load_recent_players() -> Result<Vec<Player>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_local_player_load_recent_players_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "recent players")
        }
    }

    /// Loads the local player's challengeable friends.
    pub fn load_challengable_friends() -> Result<Vec<Player>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_local_player_load_challengable_friends_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "challengeable friends")
        }
    }

    /// Fetches the identity verification signature payload.
    pub fn fetch_identity_verification_signature(
    ) -> Result<IdentityVerificationSignature, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_local_player_fetch_identity_verification_signature_json(
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "identity verification signature")
        }
    }

    /// Loads the current friend-list authorization status.
    pub fn load_friends_authorization_status() -> Result<FriendsAuthorizationStatus, GameKitError> {
        unsafe {
            let mut out_status = 0_i32;
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_local_player_load_friends_authorization_status(
                &mut out_status,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(match out_status {
                1 => FriendsAuthorizationStatus::Restricted,
                2 => FriendsAuthorizationStatus::Denied,
                3 => FriendsAuthorizationStatus::Authorized,
                _ => FriendsAuthorizationStatus::NotDetermined,
            })
        }
    }

    /// Loads the local player's friends.
    pub fn load_friends() -> Result<Vec<Player>, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_local_player_load_friends_json(&mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "friends")
        }
    }

    /// Loads friend players by scoped identifiers.
    pub fn load_friends_identified_by(ids: &[&str]) -> Result<Vec<Player>, GameKitError> {
        let ids_json = private::json_cstring(ids, "friend identifiers")?;

        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_local_player_load_friends_by_identifiers_json(
                ids_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "friends by identifiers")
        }
    }

    pub fn sign_in_view_controller() -> Result<Option<SignInViewController>, GameKitError> {
        unsafe {
            let mut out_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_local_player_sign_in_view_controller(&raw mut out_ptr, &raw mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(NonNull::new(out_ptr).map(|ptr| SignInViewController { ptr }))
        }
    }

    /// Opens the macOS friend-request creator using the current app window, when available.
    pub fn present_friend_request_creator() -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_local_player_present_friend_request(&mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
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

type AuthHandler = Mutex<Box<dyn Fn(AuthEvent) + Send + 'static>>;

/// A guard that registers an authentication handler. The handler is cleared when dropped.
pub struct AuthObserver {
    token: u64,
    context: CallbackContext<AuthHandler>,
}

impl AuthObserver {
    /// Creates a new authentication observer with the given handler.
    pub fn new<F: Fn(AuthEvent) + Send + 'static>(handler: F) -> Self {
        let context: CallbackContext<AuthHandler> =
            CallbackContext::new(Mutex::new(Box::new(handler)));
        let token = unsafe {
            ffi::gk_authenticate_handler_set(
                Some(auth_trampoline),
                context.as_ptr(),
                Some(CallbackContext::<AuthHandler>::RETAIN),
                Some(CallbackContext::<AuthHandler>::RELEASE),
            )
        };
        Self { token, context }
    }
}

impl Drop for AuthObserver {
    fn drop(&mut self) {
        self.context.deactivate();
        unsafe {
            ffi::gk_authenticate_handler_clear(self.token);
        }
    }
}

unsafe extern "C" fn auth_trampoline(refcon: *mut c_void, event_json: *const c_char) {
    if event_json.is_null() {
        return;
    }
    let Ok(event_str) = CStr::from_ptr(event_json).to_str() else {
        return;
    };
    let Ok(event) = serde_json::from_str::<AuthEvent>(event_str) else {
        return;
    };
    // SAFETY: refcon is the observer's `CallbackContext`; the Swift subscriber holds a reference
    // to it until the subscription is removed and no dispatch is using it.
    let _ = CallbackContext::<AuthHandler>::with(refcon, "auth_trampoline", |handler| {
        let handler = handler.lock().unwrap_or_else(PoisonError::into_inner);
        handler(event);
    });
}

pub struct SignInViewController {
    ptr: NonNull<c_void>,
}

impl SignInViewController {
    pub fn present(&self) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_sign_in_view_controller_present(self.ptr.as_ptr(), &raw mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Drop for SignInViewController {
    fn drop(&mut self) {
        unsafe { ffi::gk_view_controller_release(self.ptr.as_ptr()) };
    }
}

impl std::fmt::Debug for SignInViewController {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SignInViewController")
            .field("ptr", &self.ptr)
            .finish()
    }
}
