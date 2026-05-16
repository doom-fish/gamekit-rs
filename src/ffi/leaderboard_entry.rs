use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_leaderboard_load_entries_json(
        leaderboard_id: *const c_char,
        player_scope: i32,
        time_scope: i32,
        range_location: usize,
        range_length: usize,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_leaderboard_load_entries_for_players_json(
        leaderboard_id: *const c_char,
        player_ids_json: *const c_char,
        time_scope: i32,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
