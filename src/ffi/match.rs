use core::ffi::{c_char, c_void};

unsafe extern "C" {
    pub fn gk_match_retain(ptr: *mut c_void) -> *mut c_void;
    pub fn gk_match_release(ptr: *mut c_void);

    pub fn gk_match_players_json(
        ptr: *mut c_void,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_match_connected_players_json(
        ptr: *mut c_void,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_match_expected_player_count(ptr: *mut c_void) -> usize;

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

    pub fn gk_match_choose_best_hosting_player_json(
        ptr: *mut c_void,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_match_rematch(
        ptr: *mut c_void,
        out_match_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
}
