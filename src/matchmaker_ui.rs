use core::ffi::{c_char, c_void};
use std::ffi::CStr;

use serde::Deserialize;

use crate::{
    ffi, private, GameKitError, GameKitFrameworkError, Match, MatchRequest, Player,
    TurnBasedMatchRequest,
};

/// Wrapper for an accepted `GKInvite`.
#[derive(Debug)]
pub struct Invite {
    ptr: *mut c_void,
}

unsafe impl Send for Invite {}

/// Matchmaking mode exposed by the Game Center UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchmakingMode {
    Default,
    NearbyOnly,
    AutomatchOnly,
    InviteOnly,
}

/// Shared `AppKit` Game Center dialog presenter.
#[derive(Debug, Clone, Copy, Default)]
pub struct DialogController;

/// Sections that can be shown by the legacy Game Center controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCenterViewState {
    Default,
    Leaderboards,
    Achievements,
    Challenges,
    LocalPlayerProfile,
    Dashboard,
    LocalPlayerFriendsList,
}

/// Guard for a legacy Game Center controller delegate registration.
pub struct GameCenterControllerDelegate {
    controller_ptr: *mut c_void,
    handler_ptr: *mut Box<dyn Fn() + Send + 'static>,
}

unsafe impl Send for GameCenterControllerDelegate {}

/// Events emitted by `GKMatchmakerViewControllerDelegate`.
#[derive(Debug)]
pub enum MatchmakerViewControllerEvent {
    Cancelled,
    Failed { error: GameKitFrameworkError },
    FoundMatch { game_match: Match },
    FoundHostedPlayers { players: Vec<Player> },
    HostedPlayerAccepted { player: Player },
}

/// Guard for a matchmaker view controller delegate registration.
pub struct MatchmakerViewControllerDelegate {
    controller_ptr: *mut c_void,
    handler_ptr: *mut Box<dyn Fn(MatchmakerViewControllerEvent) + Send + 'static>,
}

unsafe impl Send for MatchmakerViewControllerDelegate {}

/// `AppKit` Game Center real-time matchmaking UI controller.
pub struct MatchmakerViewController {
    ptr: *mut c_void,
}

unsafe impl Send for MatchmakerViewController {}

/// Events emitted by `GKTurnBasedMatchmakerViewControllerDelegate`.
#[derive(Debug)]
pub enum TurnBasedMatchmakerViewControllerEvent {
    Cancelled,
    Failed { error: GameKitFrameworkError },
}

/// Guard for a turn-based matchmaker view controller delegate registration.
pub struct TurnBasedMatchmakerViewControllerDelegate {
    controller_ptr: *mut c_void,
    handler_ptr: *mut Box<dyn Fn(TurnBasedMatchmakerViewControllerEvent) + Send + 'static>,
}

unsafe impl Send for TurnBasedMatchmakerViewControllerDelegate {}

/// `AppKit` Game Center turn-based matchmaking UI controller.
pub struct TurnBasedMatchmakerViewController {
    ptr: *mut c_void,
}

unsafe impl Send for TurnBasedMatchmakerViewController {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MatchRequestPayload {
    min_players: u32,
    max_players: u32,
    player_group: u32,
    player_attributes: u32,
    recipient_ids: Vec<String>,
    invite_message: Option<String>,
    default_number_of_players: u32,
}

impl Invite {
    pub(crate) const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns the player who sent the invite.
    pub fn sender(&self) -> Result<Player, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_invite_sender_json(self.ptr, &mut out_json, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            private::parse_json_ptr(out_json, "invite sender")
        }
    }

    /// Returns whether the invite is for a hosted match.
    #[must_use]
    pub fn is_hosted(&self) -> bool {
        unsafe { ffi::gk_invite_is_hosted(self.ptr) }
    }

    /// Returns the invite's matchmaking player group.
    #[must_use]
    pub fn player_group(&self) -> usize {
        unsafe { ffi::gk_invite_player_group(self.ptr) }
    }

    /// Returns the invite's matchmaking player attributes.
    #[must_use]
    pub fn player_attributes(&self) -> u32 {
        unsafe { ffi::gk_invite_player_attributes(self.ptr) }
    }
}

impl Clone for Invite {
    fn clone(&self) -> Self {
        Self {
            ptr: unsafe { ffi::gk_invite_retain(self.ptr) },
        }
    }
}

impl Drop for Invite {
    fn drop(&mut self) {
        unsafe {
            ffi::gk_invite_release(self.ptr);
        }
    }
}

impl DialogController {
    /// Returns the shared Game Center dialog presenter.
    #[must_use]
    pub const fn shared() -> Self {
        Self
    }

    /// Presents a real-time matchmaking controller.
    pub fn present_matchmaker_view_controller(
        &self,
        controller: &MatchmakerViewController,
    ) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status =
                ffi::gk_dialog_present_matchmaker_view_controller(controller.ptr, &mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Presents a turn-based matchmaking controller.
    pub fn present_turn_based_matchmaker_view_controller(
        &self,
        controller: &TurnBasedMatchmakerViewController,
    ) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_dialog_present_turn_based_matchmaker_view_controller(
                controller.ptr,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Presents a legacy Game Center controller for the supplied section.
    pub fn present_game_center_state<F: Fn() + Send + 'static>(
        &self,
        state: GameCenterViewState,
        handler: F,
    ) -> Result<GameCenterControllerDelegate, GameKitError> {
        let boxed: Box<dyn Fn() + Send + 'static> = Box::new(handler);
        let handler_ptr: *mut Box<dyn Fn() + Send + 'static> = Box::into_raw(Box::new(boxed));

        unsafe {
            let mut out_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_dialog_present_game_center_view(
                state.to_raw(),
                Some(game_center_trampoline),
                handler_ptr.cast(),
                &mut out_ptr,
                &mut out_error,
            );
            if status != ffi::status::OK {
                drop(Box::from_raw(handler_ptr));
                return Err(private::error_from_status(status, out_error));
            }
            Ok(GameCenterControllerDelegate {
                controller_ptr: out_ptr,
                handler_ptr,
            })
        }
    }

    /// Presents the legacy Game Center dashboard.
    pub fn present_game_center_dashboard<F: Fn() + Send + 'static>(
        &self,
        handler: F,
    ) -> Result<GameCenterControllerDelegate, GameKitError> {
        self.present_game_center_state(GameCenterViewState::Dashboard, handler)
    }

    /// Dismisses the currently presented Game Center dialog.
    pub fn dismiss(&self) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_dialog_dismiss(&mut out_error);
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }
}

impl MatchmakerViewController {
    pub(crate) const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Creates a new matchmaking UI controller for the supplied request.
    pub fn new(request: &MatchRequest) -> Result<Self, GameKitError> {
        let payload = crate::real_time::MatchRequestPayload::from_request(request);
        let request_json = private::json_cstring(&payload, "matchmaker view controller request")?;

        unsafe {
            let mut out_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_view_controller_create(
                request_json.as_ptr(),
                &mut out_ptr,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(Self::from_raw(out_ptr))
        }
    }

    /// Creates a new matchmaking UI controller for an accepted invite.
    pub fn from_invite(invite: &Invite) -> Result<Self, GameKitError> {
        unsafe {
            let mut out_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_view_controller_create_with_invite(
                invite.ptr,
                &mut out_ptr,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(Self::from_raw(out_ptr))
        }
    }

    /// Returns the controller's current match request.
    pub fn match_request(&self) -> Result<MatchRequest, GameKitError> {
        unsafe {
            let mut out_json: *mut c_char = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_view_controller_match_request_json(
                self.ptr,
                &mut out_json,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            let payload: MatchRequestPayload =
                private::parse_json_ptr(out_json, "matchmaker view controller request")?;
            Ok(MatchRequest {
                min_players: payload.min_players,
                max_players: payload.max_players,
                player_group: payload.player_group,
                player_attributes: payload.player_attributes,
                recipient_ids: payload.recipient_ids,
                invite_message: payload.invite_message,
                default_number_of_players: Some(payload.default_number_of_players),
            })
        }
    }

    /// Returns whether the controller is configured for hosted matches.
    #[must_use]
    pub fn is_hosted(&self) -> bool {
        unsafe { ffi::gk_matchmaker_view_controller_is_hosted(self.ptr) }
    }

    /// Sets whether the controller should return hosted players instead of a peer-to-peer match.
    pub fn set_hosted(&self, hosted: bool) {
        unsafe {
            ffi::gk_matchmaker_view_controller_set_hosted(self.ptr, hosted);
        }
    }

    /// Returns the configured matchmaking mode.
    #[must_use]
    pub fn matchmaking_mode(&self) -> MatchmakingMode {
        MatchmakingMode::from_raw(unsafe {
            ffi::gk_matchmaker_view_controller_matchmaking_mode(self.ptr)
        })
    }

    /// Sets the UI matchmaking mode.
    pub fn set_matchmaking_mode(&self, mode: MatchmakingMode) {
        unsafe {
            ffi::gk_matchmaker_view_controller_set_matchmaking_mode(self.ptr, mode.to_raw());
        }
    }

    /// Returns whether the UI can start once the minimum player count is connected.
    #[must_use]
    pub fn can_start_with_minimum_players(&self) -> bool {
        unsafe { ffi::gk_matchmaker_view_controller_can_start_with_minimum_players(self.ptr) }
    }

    /// Sets whether the UI may finish once the minimum player count is connected.
    pub fn set_can_start_with_minimum_players(&self, enabled: bool) {
        unsafe {
            ffi::gk_matchmaker_view_controller_set_can_start_with_minimum_players(
                self.ptr, enabled,
            );
        }
    }

    /// Adds more players to an existing peer-to-peer match.
    pub fn add_players_to_match(&self, game_match: &Match) -> Result<(), GameKitError> {
        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_view_controller_add_players_to_match(
                self.ptr,
                game_match.ptr,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Updates the connection state for a hosted player.
    pub fn set_hosted_player_connected(
        &self,
        player: &Player,
        connected: bool,
    ) -> Result<(), GameKitError> {
        let game_player_id =
            private::cstring_from_str(&player.game_player_id, "hosted player gamePlayerID")?;

        unsafe {
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_matchmaker_view_controller_set_hosted_player_connected(
                self.ptr,
                game_player_id.as_ptr(),
                connected,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(())
        }
    }

    /// Registers a delegate callback for UI events.
    pub fn set_delegate<F: Fn(MatchmakerViewControllerEvent) + Send + 'static>(
        &self,
        handler: F,
    ) -> MatchmakerViewControllerDelegate {
        let boxed: Box<dyn Fn(MatchmakerViewControllerEvent) + Send + 'static> = Box::new(handler);
        let handler_ptr: *mut Box<dyn Fn(MatchmakerViewControllerEvent) + Send + 'static> =
            Box::into_raw(Box::new(boxed));

        unsafe {
            ffi::gk_matchmaker_view_controller_set_callbacks(
                self.ptr,
                Some(matchmaker_view_controller_trampoline),
                handler_ptr.cast(),
            );
        }

        MatchmakerViewControllerDelegate {
            controller_ptr: self.ptr,
            handler_ptr,
        }
    }

    /// Presents this controller using the shared dialog controller.
    pub fn present(&self) -> Result<(), GameKitError> {
        DialogController::shared().present_matchmaker_view_controller(self)
    }
}

impl Clone for MatchmakerViewController {
    fn clone(&self) -> Self {
        Self {
            ptr: unsafe { ffi::gk_matchmaker_view_controller_retain(self.ptr) },
        }
    }
}

impl Drop for MatchmakerViewController {
    fn drop(&mut self) {
        unsafe {
            ffi::gk_matchmaker_view_controller_release(self.ptr);
        }
    }
}

impl Drop for MatchmakerViewControllerDelegate {
    fn drop(&mut self) {
        unsafe {
            ffi::gk_matchmaker_view_controller_clear_callbacks(self.controller_ptr);
            drop(Box::from_raw(self.handler_ptr));
        }
    }
}

impl TurnBasedMatchmakerViewController {
    pub(crate) const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Creates a new turn-based matchmaking UI controller.
    pub fn new(request: &TurnBasedMatchRequest) -> Result<Self, GameKitError> {
        let payload = crate::turn_based::TurnBasedMatchRequestPayload::from_request(request);
        let request_json =
            private::json_cstring(&payload, "turn-based matchmaker view controller request")?;

        unsafe {
            let mut out_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_turn_based_matchmaker_view_controller_create(
                request_json.as_ptr(),
                &mut out_ptr,
                &mut out_error,
            );
            if status != ffi::status::OK {
                return Err(private::error_from_status(status, out_error));
            }
            Ok(Self::from_raw(out_ptr))
        }
    }

    /// Returns whether the UI shows existing matches.
    #[must_use]
    pub fn show_existing_matches(&self) -> bool {
        unsafe { ffi::gk_turn_based_matchmaker_view_controller_show_existing_matches(self.ptr) }
    }

    /// Sets whether the UI shows existing matches.
    pub fn set_show_existing_matches(&self, show_existing_matches: bool) {
        unsafe {
            ffi::gk_turn_based_matchmaker_view_controller_set_show_existing_matches(
                self.ptr,
                show_existing_matches,
            );
        }
    }

    /// Returns the configured matchmaking mode.
    #[must_use]
    pub fn matchmaking_mode(&self) -> MatchmakingMode {
        MatchmakingMode::from_raw(unsafe {
            ffi::gk_turn_based_matchmaker_view_controller_matchmaking_mode(self.ptr)
        })
    }

    /// Sets the turn-based matchmaking mode.
    pub fn set_matchmaking_mode(&self, mode: MatchmakingMode) {
        unsafe {
            ffi::gk_turn_based_matchmaker_view_controller_set_matchmaking_mode(
                self.ptr,
                mode.to_raw(),
            );
        }
    }

    /// Registers a delegate callback for UI events.
    pub fn set_delegate<F: Fn(TurnBasedMatchmakerViewControllerEvent) + Send + 'static>(
        &self,
        handler: F,
    ) -> TurnBasedMatchmakerViewControllerDelegate {
        let boxed: Box<dyn Fn(TurnBasedMatchmakerViewControllerEvent) + Send + 'static> =
            Box::new(handler);
        let handler_ptr: *mut Box<dyn Fn(TurnBasedMatchmakerViewControllerEvent) + Send + 'static> =
            Box::into_raw(Box::new(boxed));

        unsafe {
            ffi::gk_turn_based_matchmaker_view_controller_set_callbacks(
                self.ptr,
                Some(turn_based_matchmaker_view_controller_trampoline),
                handler_ptr.cast(),
            );
        }

        TurnBasedMatchmakerViewControllerDelegate {
            controller_ptr: self.ptr,
            handler_ptr,
        }
    }

    /// Presents this controller using the shared dialog controller.
    pub fn present(&self) -> Result<(), GameKitError> {
        DialogController::shared().present_turn_based_matchmaker_view_controller(self)
    }
}

impl Clone for TurnBasedMatchmakerViewController {
    fn clone(&self) -> Self {
        Self {
            ptr: unsafe { ffi::gk_turn_based_matchmaker_view_controller_retain(self.ptr) },
        }
    }
}

impl Drop for TurnBasedMatchmakerViewController {
    fn drop(&mut self) {
        unsafe {
            ffi::gk_turn_based_matchmaker_view_controller_release(self.ptr);
        }
    }
}

impl Drop for TurnBasedMatchmakerViewControllerDelegate {
    fn drop(&mut self) {
        unsafe {
            ffi::gk_turn_based_matchmaker_view_controller_clear_callbacks(self.controller_ptr);
            drop(Box::from_raw(self.handler_ptr));
        }
    }
}

impl Drop for GameCenterControllerDelegate {
    fn drop(&mut self) {
        unsafe {
            ffi::gk_game_center_controller_clear_callback(self.controller_ptr);
            ffi::gk_game_center_controller_release(self.controller_ptr);
            drop(Box::from_raw(self.handler_ptr));
        }
    }
}

impl MatchmakingMode {
    const fn from_raw(value: i32) -> Self {
        match value {
            1 => Self::NearbyOnly,
            2 => Self::AutomatchOnly,
            3 => Self::InviteOnly,
            _ => Self::Default,
        }
    }

    const fn to_raw(self) -> i32 {
        match self {
            Self::Default => 0,
            Self::NearbyOnly => 1,
            Self::AutomatchOnly => 2,
            Self::InviteOnly => 3,
        }
    }
}

impl GameCenterViewState {
    const fn to_raw(self) -> i32 {
        match self {
            Self::Default => -1,
            Self::Leaderboards => 0,
            Self::Achievements => 1,
            Self::Challenges => 2,
            Self::LocalPlayerProfile => 3,
            Self::Dashboard => 4,
            Self::LocalPlayerFriendsList => 5,
        }
    }
}

unsafe extern "C" fn game_center_trampoline(refcon: *mut c_void) {
    let handler = &*(refcon.cast::<Box<dyn Fn() + Send + 'static>>());
    handler();
}

unsafe extern "C" fn matchmaker_view_controller_trampoline(
    refcon: *mut c_void,
    kind: i32,
    payload_json: *const c_char,
    raw_ptr: *mut c_void,
) {
    let handler = &*(refcon.cast::<Box<dyn Fn(MatchmakerViewControllerEvent) + Send + 'static>>());
    match kind {
        1 => {
            if let Some(error) = parse_framework_error(payload_json) {
                handler(MatchmakerViewControllerEvent::Failed { error });
            }
        }
        2 => {
            if !raw_ptr.is_null() {
                handler(MatchmakerViewControllerEvent::FoundMatch {
                    game_match: Match::from_raw(raw_ptr),
                });
            }
        }
        3 => {
            if let Some(players) = parse_json::<Vec<Player>>(payload_json) {
                handler(MatchmakerViewControllerEvent::FoundHostedPlayers { players });
            }
        }
        4 => {
            if let Some(player) = parse_json::<Player>(payload_json) {
                handler(MatchmakerViewControllerEvent::HostedPlayerAccepted { player });
            }
        }
        _ => handler(MatchmakerViewControllerEvent::Cancelled),
    }
}

unsafe extern "C" fn turn_based_matchmaker_view_controller_trampoline(
    refcon: *mut c_void,
    kind: i32,
    payload_json: *const c_char,
) {
    let handler =
        &*(refcon.cast::<Box<dyn Fn(TurnBasedMatchmakerViewControllerEvent) + Send + 'static>>());
    match kind {
        1 => {
            if let Some(error) = parse_framework_error(payload_json) {
                handler(TurnBasedMatchmakerViewControllerEvent::Failed { error });
            }
        }
        _ => handler(TurnBasedMatchmakerViewControllerEvent::Cancelled),
    }
}

unsafe fn parse_framework_error(payload_json: *const c_char) -> Option<GameKitFrameworkError> {
    parse_json::<GameKitFrameworkError>(payload_json)
}

unsafe fn parse_json<T: for<'de> Deserialize<'de>>(payload_json: *const c_char) -> Option<T> {
    if payload_json.is_null() {
        return None;
    }

    CStr::from_ptr(payload_json)
        .to_str()
        .ok()
        .and_then(|json| serde_json::from_str::<T>(json).ok())
}
