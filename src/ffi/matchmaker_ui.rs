use core::ffi::{c_char, c_void};

unsafe extern "C" {
    pub fn gk_invite_retain(ptr: *mut c_void) -> *mut c_void;
    pub fn gk_invite_release(ptr: *mut c_void);

    pub fn gk_invite_sender_json(
        ptr: *mut c_void,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_invite_is_hosted(ptr: *mut c_void) -> bool;
    pub fn gk_invite_player_group(ptr: *mut c_void) -> usize;
    pub fn gk_invite_player_attributes(ptr: *mut c_void) -> u32;

    pub fn gk_matchmaker_view_controller_create(
        request_json: *const c_char,
        out_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_matchmaker_view_controller_create_with_invite(
        invite_ptr: *mut c_void,
        out_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_matchmaker_view_controller_retain(ptr: *mut c_void) -> *mut c_void;
    pub fn gk_matchmaker_view_controller_release(ptr: *mut c_void);

    pub fn gk_matchmaker_view_controller_match_request_json(
        ptr: *mut c_void,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_matchmaker_view_controller_is_hosted(ptr: *mut c_void) -> bool;
    pub fn gk_matchmaker_view_controller_set_hosted(ptr: *mut c_void, hosted: bool);
    pub fn gk_matchmaker_view_controller_matchmaking_mode(ptr: *mut c_void) -> i32;
    pub fn gk_matchmaker_view_controller_set_matchmaking_mode(ptr: *mut c_void, mode: i32);
    pub fn gk_matchmaker_view_controller_can_start_with_minimum_players(ptr: *mut c_void) -> bool;
    pub fn gk_matchmaker_view_controller_set_can_start_with_minimum_players(
        ptr: *mut c_void,
        enabled: bool,
    );

    pub fn gk_matchmaker_view_controller_add_players_to_match(
        ptr: *mut c_void,
        match_ptr: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_matchmaker_view_controller_set_hosted_player_connected(
        ptr: *mut c_void,
        player_game_id: *const c_char,
        connected: bool,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_matchmaker_view_controller_set_callbacks(
        ptr: *mut c_void,
        callback: Option<unsafe extern "C" fn(*mut c_void, i32, *const c_char, *mut c_void)>,
        refcon: *mut c_void,
    );

    pub fn gk_matchmaker_view_controller_clear_callbacks(ptr: *mut c_void);

    pub fn gk_turn_based_matchmaker_view_controller_create(
        request_json: *const c_char,
        out_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_turn_based_matchmaker_view_controller_retain(ptr: *mut c_void) -> *mut c_void;
    pub fn gk_turn_based_matchmaker_view_controller_release(ptr: *mut c_void);

    pub fn gk_turn_based_matchmaker_view_controller_show_existing_matches(ptr: *mut c_void)
        -> bool;
    pub fn gk_turn_based_matchmaker_view_controller_set_show_existing_matches(
        ptr: *mut c_void,
        show_existing_matches: bool,
    );

    pub fn gk_turn_based_matchmaker_view_controller_matchmaking_mode(ptr: *mut c_void) -> i32;
    pub fn gk_turn_based_matchmaker_view_controller_set_matchmaking_mode(
        ptr: *mut c_void,
        mode: i32,
    );

    pub fn gk_turn_based_matchmaker_view_controller_set_callbacks(
        ptr: *mut c_void,
        callback: Option<unsafe extern "C" fn(*mut c_void, i32, *const c_char)>,
        refcon: *mut c_void,
    );

    pub fn gk_turn_based_matchmaker_view_controller_clear_callbacks(ptr: *mut c_void);

    pub fn gk_dialog_present_matchmaker_view_controller(
        ptr: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_dialog_present_turn_based_matchmaker_view_controller(
        ptr: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_dialog_present_game_center_view(
        state: i32,
        callback: Option<unsafe extern "C" fn(*mut c_void)>,
        refcon: *mut c_void,
        out_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_game_center_controller_clear_callback(ptr: *mut c_void);
    pub fn gk_game_center_controller_release(ptr: *mut c_void);

    pub fn gk_dialog_dismiss(out_error: *mut *mut c_char) -> i32;
}
