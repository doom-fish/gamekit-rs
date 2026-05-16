use core::ffi::{c_char, c_void};

unsafe extern "C" {
    pub fn gk_game_activity_definition_load_json(
        ids_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_definition_load_achievement_descriptions_json(
        identifier: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_definition_load_leaderboards_json(
        identifier: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_definition_load_image_tiff_json(
        identifier: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_has_pending(out_pending: *mut bool, out_error: *mut *mut c_char)
        -> i32;

    pub fn gk_game_activity_valid_party_code_alphabet_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_is_valid_party_code(
        party_code: *const c_char,
        out_valid: *mut bool,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_create(
        definition_identifier: *const c_char,
        out_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_start(
        definition_identifier: *const c_char,
        out_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_start_with_party_code(
        definition_identifier: *const c_char,
        party_code: *const c_char,
        out_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_retain(ptr: *mut c_void) -> *mut c_void;
    pub fn gk_game_activity_release(ptr: *mut c_void);

    pub fn gk_game_activity_snapshot_json(
        ptr: *mut c_void,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_set_properties_json(
        ptr: *mut c_void,
        properties_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_begin(ptr: *mut c_void, out_error: *mut *mut c_char) -> i32;
    pub fn gk_game_activity_pause(ptr: *mut c_void, out_error: *mut *mut c_char) -> i32;
    pub fn gk_game_activity_resume(ptr: *mut c_void, out_error: *mut *mut c_char) -> i32;
    pub fn gk_game_activity_end(ptr: *mut c_void, out_error: *mut *mut c_char) -> i32;

    pub fn gk_game_activity_make_match_request_json(
        ptr: *mut c_void,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_find_match(
        ptr: *mut c_void,
        out_match_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_find_hosted_players_json(
        ptr: *mut c_void,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_set_score_json(
        ptr: *mut c_void,
        score_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_remove_scores(
        ptr: *mut c_void,
        leaderboard_ids_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_set_progress_json(
        ptr: *mut c_void,
        achievement_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_complete_achievement_json(
        ptr: *mut c_void,
        achievement_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_activity_remove_achievements_json(
        ptr: *mut c_void,
        achievements_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
