#![allow(missing_docs, dead_code)]
use core::ffi::{c_char, c_void};

extern "C" {
    pub fn gk_string_free(s: *mut c_char);
    
    pub fn gk_local_player_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    
    pub fn gk_authenticate_handler_set(
        callback: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
        refcon: *mut c_void,
    );
    
    pub fn gk_authenticate_handler_clear();
    
    pub fn gk_leaderboard_load_json(
        ids_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    
    pub fn gk_leaderboard_submit_score(
        score: i64,
        context: u64,
        ids_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    
    pub fn gk_leaderboard_load_entries_json(
        leaderboard_id: *const c_char,
        player_scope: i32,
        time_scope: i32,
        range_location: usize,
        range_length: usize,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    
    pub fn gk_achievement_report_json(
        achievements_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    
    pub fn gk_match_retain(ptr: *mut c_void) -> *mut c_void;
    pub fn gk_match_release(ptr: *mut c_void);
    
    pub fn gk_match_connected_players_json(
        ptr: *mut c_void,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    
    pub fn gk_match_send_data(
        ptr: *mut c_void,
        data: *const u8,
        len: usize,
        player_ids_json: *const c_char,
        mode: i32,
        out_error: *mut *mut c_char,
    ) -> i32;
    
    pub fn gk_match_send_data_to_all(
        ptr: *mut c_void,
        data: *const u8,
        len: usize,
        mode: i32,
        out_error: *mut *mut c_char,
    ) -> i32;
    
    pub fn gk_match_set_callbacks(
        ptr: *mut c_void,
        data_cb: Option<unsafe extern "C" fn(*mut c_void, *const c_char, *const u8, usize)>,
        state_cb: Option<unsafe extern "C" fn(*mut c_void, *const c_char, i32)>,
        failure_cb: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
        refcon: *mut c_void,
    );
    
    pub fn gk_match_clear_callbacks(ptr: *mut c_void);
    pub fn gk_match_disconnect(ptr: *mut c_void);
    
    pub fn gk_matchmaker_find_match_json(
        request_json: *const c_char,
        out_match_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    
    pub fn gk_matchmaker_cancel();
}

pub mod status {
    pub const OK: i32 = 0;
    pub const TIMED_OUT: i32 = -2;
    pub const NOT_AUTHENTICATED: i32 = -3;
    pub const NOT_FOUND: i32 = -5;
    pub const FRAMEWORK_ERROR: i32 = -4;
    pub const UNKNOWN: i32 = -99;
}
