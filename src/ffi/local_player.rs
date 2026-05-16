use core::ffi::{c_char, c_void};

unsafe extern "C" {
    pub fn gk_local_player_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_authenticate_handler_set(
        callback: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
        refcon: *mut c_void,
    );

    pub fn gk_authenticate_handler_clear();

    pub fn gk_local_player_load_recent_players_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_local_player_load_challengable_friends_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_local_player_fetch_identity_verification_signature_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_local_player_load_friends_authorization_status(
        out_status: *mut i32,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_local_player_load_friends_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_local_player_load_friends_by_identifiers_json(
        identifiers_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_local_player_present_friend_request(out_error: *mut *mut c_char) -> i32;
}
