use core::ffi::{c_char, c_void};

pub type AsyncJsonCb =
    extern "C" fn(json: *const c_char, error: *const c_char, ctx: *mut c_void);

pub type AsyncMatchCb =
    extern "C" fn(match_ptr: *mut c_void, error: *const c_char, ctx: *mut c_void);

unsafe extern "C" {
    /// Async authenticate — fires `cb` with local-player JSON or an error string.
    pub fn gk_local_player_authenticate_async(cb: AsyncJsonCb, ctx: *mut c_void);

    /// Async load friends authorization status — fires `cb` with the raw status
    /// integer encoded as a decimal string (e.g. `"3"`).
    pub fn gk_local_player_load_friends_authorization_async(
        cb: AsyncJsonCb,
        ctx: *mut c_void,
    );

    /// Async findMatch — fires `cb` with a retained `GKMatchBox` pointer on success.
    pub fn gk_matchmaker_find_match_async(
        request_json: *const c_char,
        cb: AsyncMatchCb,
        ctx: *mut c_void,
    );

    /// Async findPlayers (hosted) — fires `cb` with a JSON array of player objects.
    pub fn gk_matchmaker_find_players_async(
        request_json: *const c_char,
        cb: AsyncJsonCb,
        ctx: *mut c_void,
    );

    /// Async loadLeaderboards — fires `cb` with a JSON array of leaderboard objects.
    pub fn gk_leaderboard_load_async(
        ids_json: *const c_char,
        cb: AsyncJsonCb,
        ctx: *mut c_void,
    );

    /// Async loadEntries — fires `cb` with a `GKLoadEntriesPayload` JSON object.
    pub fn gk_leaderboard_load_entries_async(
        leaderboard_id: *const c_char,
        player_scope: i32,
        time_scope: i32,
        range_location: usize,
        range_length: usize,
        cb: AsyncJsonCb,
        ctx: *mut c_void,
    );

    /// Async loadAchievements — fires `cb` with a JSON array of achievement objects.
    pub fn gk_achievement_load_async(cb: AsyncJsonCb, ctx: *mut c_void);

    /// Async report achievements — fires `cb` with `"null"` or an error string.
    pub fn gk_achievement_report_async(
        achievements_json: *const c_char,
        cb: AsyncJsonCb,
        ctx: *mut c_void,
    );

    /// Async fetchSavedGames — fires `cb` with a JSON array of saved-game objects.
    pub fn gk_saved_game_fetch_all_async(cb: AsyncJsonCb, ctx: *mut c_void);

    /// Async loadData for a saved game — fires `cb` with a `GKBinaryPayload` JSON.
    pub fn gk_saved_game_load_data_async(
        saved_game_json: *const c_char,
        cb: AsyncJsonCb,
        ctx: *mut c_void,
    );

    /// Async saveGameData — fires `cb` with the new saved-game JSON.
    pub fn gk_saved_game_save_async(
        name: *const c_char,
        data: *const u8,
        len: usize,
        cb: AsyncJsonCb,
        ctx: *mut c_void,
    );
}
