use core::ffi::{c_char, c_void};

unsafe extern "C" {
    pub fn gk_local_player_listener_register(
        callback: Option<unsafe extern "C" fn(*mut c_void, i32, *const c_char, *mut c_void) -> i32>,
        refcon: *mut c_void,
        out_listener_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_local_player_listener_unregister(ptr: *mut c_void);
    pub fn gk_local_player_unregister_all_listeners();
}
