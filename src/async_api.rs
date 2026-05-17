//! Async API for `GameKit`
//!
//! This module provides executor-agnostic `Future` wrappers for `GameKit`'s
//! completion-handler APIs.  Enable the `async` Cargo feature to activate it.
//!
//! ## Available types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`AsyncLocalPlayer`] | Authenticate and query friends-authorization |
//! | [`AsyncMatchmaker`] | Find real-time matches and hosted players |
//! | [`AsyncLeaderboard`] | Load leaderboards and their entries |
//! | [`AsyncAchievement`] | Load and report achievements |
//! | [`AsyncSavedGame`] | Fetch, load, and save game data |
//!
//! ## Runtime-agnostic design
//!
//! All futures use only `std` types (`Arc`, `Mutex`, `Waker`) and work with
//! any async executor — Tokio, async-std, smol, or `pollster`.
//!
//! ## Example
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use gamekit::async_api::{AsyncAchievement, AsyncLeaderboard};
//!
//! let achievements = AsyncAchievement::load().await?;
//! println!("loaded {} achievements", achievements.len());
//!
//! let leaderboards = AsyncLeaderboard::load(&[])?.await?;
//! println!("loaded {} leaderboards", leaderboards.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Tier-2 note
//!
//! The following `GameKit` surfaces fire callbacks more than once and are
//! **not** covered here (use a `Stream`-based Tier-2 module instead):
//! - `GKMatchDelegate` (data / state / failure callbacks)
//! - `GKLocalPlayerListener` (turn-based, invite, saved-game conflict events)
//! - `GKTurnBasedEventListener`

use std::ffi::{c_void, CStr, CString};
use std::future::Future;
use std::ops::Range;
use std::pin::Pin;
use std::task::{Context, Poll};

use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};

use crate::achievement::{Achievement, AchievementPayload};
use crate::error::GameKitError;
use crate::ffi::{self, AsyncJsonCb, AsyncMatchCb};
use crate::leaderboard::{Leaderboard, LeaderboardPayload, PlayerScope, TimeScope};
use crate::leaderboard_entry::{LeaderboardEntry, LoadEntriesResult};
use crate::local_player::{FriendsAuthorizationStatus, LocalPlayer};
use crate::player::Player;
use crate::private;
use crate::r#match::Match;
use crate::real_time::{request_json, MatchRequest};
use crate::save::SavedGame;

// ============================================================================
// Internal helpers
// ============================================================================

/// Resolve a JSON C-string to a Rust type, mapping errors to `GameKitError`.
#[allow(dead_code)]
fn parse_json<T: serde::de::DeserializeOwned>(
    json: *const i8,
    label: &str,
) -> Result<T, GameKitError> {
    if json.is_null() {
        return Err(GameKitError::Unknown(format!("null JSON for {label}")));
    }
    let s = unsafe { CStr::from_ptr(json).to_string_lossy() };
    serde_json::from_str(&s).map_err(|e| GameKitError::Unknown(format!("bad {label} JSON: {e}")))
}

/// Build a nul-terminated `CString` from a `&str`, mapping errors to `GameKitError`.
fn cstr(s: &str, label: &str) -> Result<CString, GameKitError> {
    CString::new(s).map_err(|_| GameKitError::Unknown(format!("nul byte in {label}")))
}

// ============================================================================
// Newtype wrapper that is `Send` for raw match pointers
// ============================================================================

/// A `usize`-wrapped `*mut c_void` that is `Send`.  Used so that
/// `AsyncCompletion<MatchPtrSend>` can be polled across thread boundaries.
struct MatchPtrSend(usize);
/// SAFETY: The wrapped pointer is a `GKMatch` Obj-C object which the `GameKit`
/// runtime retains for us; it is safe to move between threads.
unsafe impl Send for MatchPtrSend {}

// ============================================================================
// AsyncLocalPlayer
// ============================================================================

/// Callback for authenticate / friends-auth-status futures
extern "C" fn local_player_json_cb(json: *const i8, error: *const i8, ctx: *mut c_void) {
    if !error.is_null() {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<String>::complete_err(ctx, msg) };
    } else if !json.is_null() {
        let s = unsafe { CStr::from_ptr(json).to_string_lossy().into_owned() };
        unsafe { AsyncCompletion::complete_ok(ctx, s) };
    } else {
        unsafe { AsyncCompletion::<String>::complete_err(ctx, "Unknown error".into()) };
    }
}

// --- authenticate -----------------------------------------------------------

/// Future produced by [`AsyncLocalPlayer::authenticate`].
pub struct AuthenticateFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for AuthenticateFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthenticateFuture").finish_non_exhaustive()
    }
}

impl Future for AuthenticateFuture {
    type Output = Result<LocalPlayer, GameKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            r.map_err(GameKitError::Unknown)
                .and_then(|json| parse_local_player_from_json(&json))
        })
    }
}

fn parse_local_player_from_json(json: &str) -> Result<LocalPlayer, GameKitError> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[allow(clippy::struct_excessive_bools)]
    struct Payload {
        is_authenticated: bool,
        is_underage: bool,
        is_multiplayer_gaming_restricted: bool,
        is_personalized_communication_restricted: bool,
        is_presenting_friend_request_view_controller: bool,
        player: Player,
    }
    let p: Payload = serde_json::from_str(json)
        .map_err(|e| GameKitError::Unknown(format!("bad local player JSON: {e}")))?;
    Ok(LocalPlayer {
        is_authenticated: p.is_authenticated,
        is_underage: p.is_underage,
        is_multiplayer_gaming_restricted: p.is_multiplayer_gaming_restricted,
        is_personalized_communication_restricted: p.is_personalized_communication_restricted,
        is_presenting_friend_request_view_controller: p
            .is_presenting_friend_request_view_controller,
        player: p.player,
    })
}

// --- friends authorization --------------------------------------------------

/// Future produced by [`AsyncLocalPlayer::load_friends_authorization`].
pub struct FriendsAuthorizationFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for FriendsAuthorizationFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FriendsAuthorizationFuture")
            .finish_non_exhaustive()
    }
}

impl Future for FriendsAuthorizationFuture {
    type Output = Result<FriendsAuthorizationStatus, GameKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            r.map_err(GameKitError::Unknown).and_then(|s| {
                let raw: i32 = s
                    .parse()
                    .map_err(|_| GameKitError::Unknown(format!("bad auth status: {s}")))?;
                Ok(friends_auth_status_from_raw(raw))
            })
        })
    }
}

const fn friends_auth_status_from_raw(raw: i32) -> FriendsAuthorizationStatus {
    match raw {
        1 => FriendsAuthorizationStatus::Restricted,
        2 => FriendsAuthorizationStatus::Denied,
        3 => FriendsAuthorizationStatus::Authorized,
        _ => FriendsAuthorizationStatus::NotDetermined,
    }
}

// --- public API -------------------------------------------------------------

/// Async wrappers for [`LocalPlayer`] operations.
#[derive(Debug, Clone, Copy)]
pub struct AsyncLocalPlayer;

impl AsyncLocalPlayer {
    /// Authenticate the local player with Game Center.
    ///
    /// On macOS 13+ this calls `GKLocalPlayer.local.authenticate()`.
    /// On older systems the current (possibly unauthenticated) snapshot is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails (e.g. not signed in to iCloud).
    pub fn authenticate() -> AuthenticateFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe { ffi::gk_local_player_authenticate_async(local_player_json_cb as AsyncJsonCb, ctx) };
        AuthenticateFuture { inner: future }
    }

    /// Load the friends-list authorization status asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if the framework call fails.
    pub fn load_friends_authorization() -> FriendsAuthorizationFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::gk_local_player_load_friends_authorization_async(
                local_player_json_cb as AsyncJsonCb,
                ctx,
            );
        };
        FriendsAuthorizationFuture { inner: future }
    }
}

// ============================================================================
// AsyncMatchmaker
// ============================================================================

// --- find_match -------------------------------------------------------------

extern "C" fn find_match_cb(ptr: *mut c_void, error: *const i8, ctx: *mut c_void) {
    if !error.is_null() {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<MatchPtrSend>::complete_err(ctx, msg) };
    } else if !ptr.is_null() {
        unsafe { AsyncCompletion::complete_ok(ctx, MatchPtrSend(ptr as usize)) };
    } else {
        unsafe {
            AsyncCompletion::<MatchPtrSend>::complete_err(ctx, "nil match pointer".into());
        };
    }
}

/// Future produced by [`AsyncMatchmaker::find_match`].
pub struct FindMatchFuture {
    inner: AsyncCompletionFuture<MatchPtrSend>,
}

impl std::fmt::Debug for FindMatchFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FindMatchFuture").finish_non_exhaustive()
    }
}

impl Future for FindMatchFuture {
    type Output = Result<Match, GameKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            r.map_err(GameKitError::Unknown)
                .map(|p| Match::from_raw(p.0 as *mut c_void))
        })
    }
}

// --- find_players -----------------------------------------------------------

extern "C" fn json_cb(json: *const i8, error: *const i8, ctx: *mut c_void) {
    if !error.is_null() {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<String>::complete_err(ctx, msg) };
    } else if !json.is_null() {
        let s = unsafe { CStr::from_ptr(json).to_string_lossy().into_owned() };
        unsafe { AsyncCompletion::complete_ok(ctx, s) };
    } else {
        unsafe { AsyncCompletion::<String>::complete_err(ctx, "Unknown error".into()) };
    }
}

/// Future produced by [`AsyncMatchmaker::find_players`].
pub struct FindPlayersFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for FindPlayersFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FindPlayersFuture").finish_non_exhaustive()
    }
}

impl Future for FindPlayersFuture {
    type Output = Result<Vec<Player>, GameKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            r.map_err(GameKitError::Unknown)
                .and_then(|json| {
                    serde_json::from_str(&json)
                        .map_err(|e| GameKitError::Unknown(format!("bad players JSON: {e}")))
                })
        })
    }
}

/// Async wrappers for matchmaking operations.
#[derive(Debug, Clone, Copy)]
pub struct AsyncMatchmaker;

impl AsyncMatchmaker {
    /// Find a peer-to-peer real-time match asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if matchmaking fails or is cancelled.
    pub fn find_match(request: &MatchRequest) -> Result<FindMatchFuture, GameKitError> {
        let json = request_json(request)?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::gk_matchmaker_find_match_async(
                json.as_ptr(),
                find_match_cb as AsyncMatchCb,
                ctx,
            );
        };
        Ok(FindMatchFuture { inner: future })
    }

    /// Find hosted players for a real-time match asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if the request serialization or matchmaking fails.
    pub fn find_players(request: &MatchRequest) -> Result<FindPlayersFuture, GameKitError> {
        let json = request_json(request)?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::gk_matchmaker_find_players_async(json.as_ptr(), json_cb as AsyncJsonCb, ctx);
        };
        Ok(FindPlayersFuture { inner: future })
    }
}

// ============================================================================
// AsyncLeaderboard
// ============================================================================

/// Future produced by [`AsyncLeaderboard::load`].
pub struct LoadLeaderboardsFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for LoadLeaderboardsFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadLeaderboardsFuture")
            .finish_non_exhaustive()
    }
}

impl Future for LoadLeaderboardsFuture {
    type Output = Result<Vec<Leaderboard>, GameKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            r.map_err(GameKitError::Unknown).and_then(|json| {
                let payloads: Vec<LeaderboardPayload> = serde_json::from_str(&json)
                    .map_err(|e| GameKitError::Unknown(format!("bad leaderboards JSON: {e}")))?;
                Ok(payloads.into_iter().map(Leaderboard::from_payload).collect())
            })
        })
    }
}

/// Future produced by [`AsyncLeaderboard::load_entries`].
pub struct LoadEntriesFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for LoadEntriesFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadEntriesFuture").finish_non_exhaustive()
    }
}

impl Future for LoadEntriesFuture {
    type Output = Result<LoadEntriesResult, GameKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            r.map_err(GameKitError::Unknown).and_then(|json| {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct EntryPayload {
                    rank: i64,
                    score: i64,
                    formatted_score: String,
                    context: u64,
                    date: String,
                    player: Player,
                }
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct ResultPayload {
                    local_player_entry: Option<EntryPayload>,
                    entries: Vec<EntryPayload>,
                    total_player_count: i64,
                }
                fn entry_from(p: EntryPayload) -> LeaderboardEntry {
                    LeaderboardEntry {
                        rank: p.rank,
                        score: p.score,
                        formatted_score: p.formatted_score,
                        context: p.context,
                        date: p.date,
                        player: p.player,
                    }
                }
                let payload: ResultPayload = serde_json::from_str(&json)
                    .map_err(|e| GameKitError::Unknown(format!("bad entries JSON: {e}")))?;
                Ok(LoadEntriesResult {
                    local_player_entry: payload.local_player_entry.map(entry_from),
                    entries: payload.entries.into_iter().map(entry_from).collect(),
                    total_player_count: payload.total_player_count,
                })
            })
        })
    }
}

/// Async wrappers for leaderboard operations.
#[derive(Debug, Clone, Copy)]
pub struct AsyncLeaderboard;

impl AsyncLeaderboard {
    /// Load leaderboards by ID (empty slice = all leaderboards).
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the framework call fails.
    pub fn load(ids: &[&str]) -> Result<LoadLeaderboardsFuture, GameKitError> {
        let (future, ctx) = AsyncCompletion::create();
        if ids.is_empty() {
            unsafe {
                ffi::gk_leaderboard_load_async(std::ptr::null(), json_cb as AsyncJsonCb, ctx);
            };
        } else {
            let json = private::json_cstring(ids, "leaderboard IDs")?;
            unsafe {
                ffi::gk_leaderboard_load_async(json.as_ptr(), json_cb as AsyncJsonCb, ctx);
            };
        }
        Ok(LoadLeaderboardsFuture { inner: future })
    }

    /// Load entries for a leaderboard asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the framework call fails.
    pub fn load_entries(
        leaderboard_id: &str,
        player_scope: PlayerScope,
        time_scope: TimeScope,
        range: Range<usize>,
    ) -> Result<LoadEntriesFuture, GameKitError> {
        let id_cstr = cstr(leaderboard_id, "leaderboard ID")?;
        let ps = match player_scope {
            PlayerScope::Global => 0,
            PlayerScope::FriendsOnly => 1,
        };
        let ts = match time_scope {
            TimeScope::Today => 0,
            TimeScope::Week => 1,
            TimeScope::AllTime => 2,
        };
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::gk_leaderboard_load_entries_async(
                id_cstr.as_ptr(),
                ps,
                ts,
                range.start,
                range.end.saturating_sub(range.start),
                json_cb as AsyncJsonCb,
                ctx,
            );
        };
        Ok(LoadEntriesFuture { inner: future })
    }
}

// ============================================================================
// AsyncAchievement
// ============================================================================

/// Future produced by [`AsyncAchievement::load`].
pub struct LoadAchievementsFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for LoadAchievementsFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadAchievementsFuture")
            .finish_non_exhaustive()
    }
}

impl Future for LoadAchievementsFuture {
    type Output = Result<Vec<Achievement>, GameKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            r.map_err(GameKitError::Unknown).and_then(|json| {
                let payloads: Vec<AchievementPayload> = serde_json::from_str(&json)
                    .map_err(|e| GameKitError::Unknown(format!("bad achievements JSON: {e}")))?;
                Ok(payloads
                    .into_iter()
                    .map(Achievement::from_payload)
                    .collect())
            })
        })
    }
}

/// Future produced by [`AsyncAchievement::report`].
pub struct ReportAchievementFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for ReportAchievementFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReportAchievementFuture")
            .finish_non_exhaustive()
    }
}

impl Future for ReportAchievementFuture {
    type Output = Result<(), GameKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map(|_| ()).map_err(GameKitError::Unknown))
    }
}

/// Async wrappers for achievement operations.
#[derive(Debug, Clone, Copy)]
pub struct AsyncAchievement;

impl AsyncAchievement {
    /// Load all achievements for the local player asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if the framework call fails.
    pub fn load() -> LoadAchievementsFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe { ffi::gk_achievement_load_async(json_cb as AsyncJsonCb, ctx) };
        LoadAchievementsFuture { inner: future }
    }

    /// Report a batch of achievements asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the framework call fails.
    pub fn report(achievements: &[Achievement]) -> Result<ReportAchievementFuture, GameKitError> {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Input<'a> {
            identifier: &'a str,
            percent_complete: f64,
            shows_completion_banner: bool,
            player_game_id: Option<String>,
        }
        let inputs: Vec<Input<'_>> = achievements
            .iter()
            .map(|a| Input {
                identifier: &a.identifier,
                percent_complete: a.percent_complete,
                shows_completion_banner: a.shows_completion_banner,
                player_game_id: a.player.as_ref().map(|p| p.game_player_id.clone()),
            })
            .collect();
        let json = private::json_cstring(&inputs, "achievements")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::gk_achievement_report_async(json.as_ptr(), json_cb as AsyncJsonCb, ctx);
        };
        Ok(ReportAchievementFuture { inner: future })
    }
}

// ============================================================================
// AsyncSavedGame
// ============================================================================

/// Future produced by [`AsyncSavedGame::fetch_all`].
pub struct FetchAllSavedGamesFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for FetchAllSavedGamesFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FetchAllSavedGamesFuture")
            .finish_non_exhaustive()
    }
}

impl Future for FetchAllSavedGamesFuture {
    type Output = Result<Vec<SavedGame>, GameKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            r.map_err(GameKitError::Unknown).and_then(|json| {
                serde_json::from_str(&json)
                    .map_err(|e| GameKitError::Unknown(format!("bad saved games JSON: {e}")))
            })
        })
    }
}

/// Future produced by [`AsyncSavedGame::load_data`].
pub struct LoadSavedGameDataFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for LoadSavedGameDataFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadSavedGameDataFuture")
            .finish_non_exhaustive()
    }
}

impl Future for LoadSavedGameDataFuture {
    type Output = Result<Vec<u8>, GameKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            r.map_err(GameKitError::Unknown).and_then(|json| {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Payload {
                    data_base64: String,
                }
                let p: Payload = serde_json::from_str(&json).map_err(|e| {
                    GameKitError::Unknown(format!("bad saved-game data JSON: {e}"))
                })?;
                base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    &p.data_base64,
                )
                .map_err(|e| GameKitError::Unknown(format!("base64 decode error: {e}")))
            })
        })
    }
}

/// Future produced by [`AsyncSavedGame::save`].
pub struct SaveGameFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for SaveGameFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SaveGameFuture").finish_non_exhaustive()
    }
}

impl Future for SaveGameFuture {
    type Output = Result<SavedGame, GameKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            r.map_err(GameKitError::Unknown).and_then(|json| {
                serde_json::from_str(&json)
                    .map_err(|e| GameKitError::Unknown(format!("bad saved-game JSON: {e}")))
            })
        })
    }
}

/// Async wrappers for saved-game operations.
#[derive(Debug, Clone, Copy)]
pub struct AsyncSavedGame;

impl AsyncSavedGame {
    /// Fetch all saved games for the local player asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if the framework call fails.
    pub fn fetch_all() -> FetchAllSavedGamesFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe { ffi::gk_saved_game_fetch_all_async(json_cb as AsyncJsonCb, ctx) };
        FetchAllSavedGamesFuture { inner: future }
    }

    /// Load the binary data for a saved game asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if the saved game is not found or data loading fails.
    pub fn load_data(saved_game: &SavedGame) -> Result<LoadSavedGameDataFuture, GameKitError> {
        let json = private::json_cstring(saved_game, "saved game")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::gk_saved_game_load_data_async(json.as_ptr(), json_cb as AsyncJsonCb, ctx);
        };
        Ok(LoadSavedGameDataFuture { inner: future })
    }

    /// Save binary game data asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if the name contains a nul byte or the framework call fails.
    pub fn save(name: &str, data: &[u8]) -> Result<SaveGameFuture, GameKitError> {
        let name_cstr = cstr(name, "saved game name")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::gk_saved_game_save_async(
                name_cstr.as_ptr(),
                data.as_ptr(),
                data.len(),
                json_cb as AsyncJsonCb,
                ctx,
            );
        };
        Ok(SaveGameFuture { inner: future })
    }
}
