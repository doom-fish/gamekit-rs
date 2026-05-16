use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_saved_games_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_saved_game_load_data_json(
        saved_game_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_saved_game_save_json(
        name: *const c_char,
        data: *const u8,
        len: usize,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_saved_game_delete(name: *const c_char, out_error: *mut *mut c_char) -> i32;

    pub fn gk_saved_game_resolve_conflicts_json(
        saved_games_json: *const c_char,
        data: *const u8,
        len: usize,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
