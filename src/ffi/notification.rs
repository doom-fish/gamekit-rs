use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_notification_show(title: *const c_char, message: *const c_char);
    pub fn gk_notification_show_with_duration(
        title: *const c_char,
        message: *const c_char,
        duration_seconds: f64,
    );
}
