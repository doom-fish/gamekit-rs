use core::ffi::{c_char, c_void};

unsafe extern "C" {
    pub fn gk_access_point_json(out_json: *mut *mut c_char, out_error: *mut *mut c_char) -> i32;

    pub fn gk_access_point_set_active(active: bool);
    pub fn gk_access_point_set_location(location: i32);

    pub fn gk_access_point_trigger_target(
        kind: i32,
        state: i32,
        identifier: *const c_char,
        player_scope: i32,
        time_scope: i32,
        activity: *mut c_void,
        callback: Option<unsafe extern "C" fn(*mut c_void, i32, *mut c_char)>,
        refcon: *mut c_void,
        discard: Option<unsafe extern "C" fn(*mut c_void)>,
        out_error: *mut *mut c_char,
    ) -> i32;
}
