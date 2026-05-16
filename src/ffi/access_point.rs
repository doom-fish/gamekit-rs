use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_access_point_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_access_point_set_active(active: bool);
    pub fn gk_access_point_set_location(location: i32);

    pub fn gk_access_point_trigger(out_error: *mut *mut c_char) -> i32;
    pub fn gk_access_point_trigger_state(state: i32, out_error: *mut *mut c_char) -> i32;
}
