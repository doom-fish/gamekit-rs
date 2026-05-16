use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_turn_based_find_match_json(
        request_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_load_matches_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_load_match_json(
        match_id: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_rematch_json(
        match_id: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_accept_invite_json(
        match_id: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_decline_invite(
        match_id: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_remove(match_id: *const c_char, out_error: *mut *mut c_char) -> i32;

    pub fn gk_turn_based_load_match_data_json(
        match_id: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_save_current_turn(
        match_id: *const c_char,
        data: *const u8,
        len: usize,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_end_turn(
        match_id: *const c_char,
        next_indices_json: *const c_char,
        timeout_seconds: f64,
        data: *const u8,
        len: usize,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_participant_quit_in_turn(
        match_id: *const c_char,
        outcome: i32,
        next_indices_json: *const c_char,
        timeout_seconds: f64,
        data: *const u8,
        len: usize,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_participant_quit_out_of_turn(
        match_id: *const c_char,
        outcome: i32,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_end_match_in_turn(
        match_id: *const c_char,
        data: *const u8,
        len: usize,
        scores_json: *const c_char,
        achievements_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_save_merged_match_data(
        match_id: *const c_char,
        data: *const u8,
        len: usize,
        resolved_indices_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_send_exchange_json(
        match_id: *const c_char,
        participant_indices_json: *const c_char,
        data: *const u8,
        len: usize,
        key: *const c_char,
        args_json: *const c_char,
        timeout_seconds: f64,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_cancel_exchange(
        match_id: *const c_char,
        exchange_index: usize,
        key: *const c_char,
        args_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_reply_exchange(
        match_id: *const c_char,
        exchange_index: usize,
        key: *const c_char,
        args_json: *const c_char,
        data: *const u8,
        len: usize,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_send_reminder(
        match_id: *const c_char,
        participant_indices_json: *const c_char,
        key: *const c_char,
        args_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
