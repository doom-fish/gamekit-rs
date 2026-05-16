use core::ffi::{c_char, c_void};
use std::ffi::CStr;

use serde::Deserialize;

use crate::{
    ffi, private, GameActivitySnapshot, GameKitError, Invite, LocalPlayer, Player, SavedGame,
    TurnBasedExchange, TurnBasedExchangeReply, TurnBasedMatch,
};

/// Events delivered through `GKLocalPlayerListener`.
#[derive(Debug)]
pub enum LocalPlayerEvent {
    AcceptedInvite {
        player: Player,
        invite: Invite,
    },
    RequestedMatchWithRecipients {
        player: Player,
        recipients: Vec<Player>,
    },
    RequestedTurnBasedMatchWithOtherPlayers {
        player: Player,
        players_to_invite: Vec<Player>,
    },
    ReceivedTurnEvent {
        player: Player,
        match_snapshot: TurnBasedMatch,
        did_become_active: bool,
    },
    MatchEnded {
        player: Player,
        match_snapshot: TurnBasedMatch,
    },
    ReceivedExchangeRequest {
        player: Player,
        exchange: TurnBasedExchange,
        match_snapshot: TurnBasedMatch,
    },
    ReceivedExchangeCancellation {
        player: Player,
        exchange: TurnBasedExchange,
        match_snapshot: TurnBasedMatch,
    },
    ReceivedExchangeReplies {
        player: Player,
        replies: Vec<TurnBasedExchangeReply>,
        completed_exchange: TurnBasedExchange,
        match_snapshot: TurnBasedMatch,
    },
    WantsToQuitMatch {
        player: Player,
        match_snapshot: TurnBasedMatch,
    },
    ModifiedSavedGame {
        player: Player,
        saved_game: SavedGame,
    },
    ConflictingSavedGames {
        player: Player,
        saved_games: Vec<SavedGame>,
    },
    WantsToPlayGameActivity {
        player: Player,
        activity: GameActivitySnapshot,
    },
}

/// Guard that keeps a `GKLocalPlayerListener` registration alive.
pub struct LocalPlayerListener {
    listener_ptr: *mut c_void,
    handler_ptr: *mut Box<dyn Fn(LocalPlayerEvent) -> bool + Send + 'static>,
}

unsafe impl Send for LocalPlayerListener {}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum LocalPlayerEventPayload {
    AcceptedInvite {
        player: Player,
    },
    RequestedMatchWithRecipients {
        player: Player,
        recipients: Vec<Player>,
    },
    RequestedTurnBasedMatchWithOtherPlayers {
        player: Player,
        players_to_invite: Vec<Player>,
    },
    ReceivedTurnEvent {
        player: Player,
        match_snapshot: crate::turn_based::TurnBasedMatchPayload,
        did_become_active: bool,
    },
    MatchEnded {
        player: Player,
        match_snapshot: crate::turn_based::TurnBasedMatchPayload,
    },
    ReceivedExchangeRequest {
        player: Player,
        exchange: crate::turn_based::TurnBasedExchangePayload,
        match_snapshot: crate::turn_based::TurnBasedMatchPayload,
    },
    ReceivedExchangeCancellation {
        player: Player,
        exchange: crate::turn_based::TurnBasedExchangePayload,
        match_snapshot: crate::turn_based::TurnBasedMatchPayload,
    },
    ReceivedExchangeReplies {
        player: Player,
        replies: Vec<crate::turn_based::TurnBasedExchangeReplyPayload>,
        completed_exchange: crate::turn_based::TurnBasedExchangePayload,
        match_snapshot: crate::turn_based::TurnBasedMatchPayload,
    },
    WantsToQuitMatch {
        player: Player,
        match_snapshot: crate::turn_based::TurnBasedMatchPayload,
    },
    ModifiedSavedGame {
        player: Player,
        saved_game: SavedGame,
    },
    ConflictingSavedGames {
        player: Player,
        saved_games: Vec<SavedGame>,
    },
    WantsToPlayGameActivity {
        player: Player,
        activity: crate::game_activity::GameActivitySnapshotPayload,
    },
}

impl LocalPlayer {
    /// Registers a local-player listener. The closure's return value is only used for
    /// `WantsToPlayGameActivity` events, where `true` marks the activity as handled.
    pub fn register_listener<F: Fn(LocalPlayerEvent) -> bool + Send + 'static>(
        handler: F,
    ) -> Result<LocalPlayerListener, GameKitError> {
        let boxed: Box<dyn Fn(LocalPlayerEvent) -> bool + Send + 'static> = Box::new(handler);
        let handler_ptr: *mut Box<dyn Fn(LocalPlayerEvent) -> bool + Send + 'static> =
            Box::into_raw(Box::new(boxed));

        unsafe {
            let mut out_listener_ptr: *mut c_void = std::ptr::null_mut();
            let mut out_error: *mut c_char = std::ptr::null_mut();
            let status = ffi::gk_local_player_listener_register(
                Some(local_player_listener_trampoline),
                handler_ptr.cast(),
                &mut out_listener_ptr,
                &mut out_error,
            );
            if status != ffi::status::OK {
                drop(Box::from_raw(handler_ptr));
                return Err(private::error_from_status(status, out_error));
            }

            Ok(LocalPlayerListener {
                listener_ptr: out_listener_ptr,
                handler_ptr,
            })
        }
    }

    /// Unregisters every listener currently registered with `GKLocalPlayer.local`.
    pub fn unregister_all_listeners() {
        unsafe {
            ffi::gk_local_player_unregister_all_listeners();
        }
    }
}

impl Drop for LocalPlayerListener {
    fn drop(&mut self) {
        unsafe {
            ffi::gk_local_player_listener_unregister(self.listener_ptr);
            drop(Box::from_raw(self.handler_ptr));
        }
    }
}

unsafe extern "C" fn local_player_listener_trampoline(
    refcon: *mut c_void,
    _kind: i32,
    payload_json: *const c_char,
    raw_ptr: *mut c_void,
) -> i32 {
    let handler = &*(refcon.cast::<Box<dyn Fn(LocalPlayerEvent) -> bool + Send + 'static>>());

    let Some(payload) = parse_event_payload(payload_json) else {
        return 0;
    };
    let Some(event) = event_from_payload(payload, raw_ptr) else {
        return 0;
    };

    i32::from(handler(event))
}

unsafe fn event_from_payload(
    payload: LocalPlayerEventPayload,
    raw_ptr: *mut c_void,
) -> Option<LocalPlayerEvent> {
    Some(match payload {
        LocalPlayerEventPayload::AcceptedInvite { player } => {
            if raw_ptr.is_null() {
                return None;
            }
            LocalPlayerEvent::AcceptedInvite {
                player,
                invite: Invite::from_raw(raw_ptr),
            }
        }
        LocalPlayerEventPayload::RequestedMatchWithRecipients { player, recipients } => {
            LocalPlayerEvent::RequestedMatchWithRecipients { player, recipients }
        }
        LocalPlayerEventPayload::RequestedTurnBasedMatchWithOtherPlayers {
            player,
            players_to_invite,
        } => LocalPlayerEvent::RequestedTurnBasedMatchWithOtherPlayers {
            player,
            players_to_invite,
        },
        LocalPlayerEventPayload::ReceivedTurnEvent {
            player,
            match_snapshot,
            did_become_active,
        } => LocalPlayerEvent::ReceivedTurnEvent {
            player,
            match_snapshot: TurnBasedMatch::from_payload(match_snapshot),
            did_become_active,
        },
        LocalPlayerEventPayload::MatchEnded {
            player,
            match_snapshot,
        } => LocalPlayerEvent::MatchEnded {
            player,
            match_snapshot: TurnBasedMatch::from_payload(match_snapshot),
        },
        LocalPlayerEventPayload::ReceivedExchangeRequest {
            player,
            exchange,
            match_snapshot,
        } => LocalPlayerEvent::ReceivedExchangeRequest {
            player,
            exchange: TurnBasedExchange::from_payload(exchange),
            match_snapshot: TurnBasedMatch::from_payload(match_snapshot),
        },
        LocalPlayerEventPayload::ReceivedExchangeCancellation {
            player,
            exchange,
            match_snapshot,
        } => LocalPlayerEvent::ReceivedExchangeCancellation {
            player,
            exchange: TurnBasedExchange::from_payload(exchange),
            match_snapshot: TurnBasedMatch::from_payload(match_snapshot),
        },
        LocalPlayerEventPayload::ReceivedExchangeReplies {
            player,
            replies,
            completed_exchange,
            match_snapshot,
        } => LocalPlayerEvent::ReceivedExchangeReplies {
            player,
            replies: replies
                .into_iter()
                .map(TurnBasedExchangeReply::from_payload)
                .collect(),
            completed_exchange: TurnBasedExchange::from_payload(completed_exchange),
            match_snapshot: TurnBasedMatch::from_payload(match_snapshot),
        },
        LocalPlayerEventPayload::WantsToQuitMatch {
            player,
            match_snapshot,
        } => LocalPlayerEvent::WantsToQuitMatch {
            player,
            match_snapshot: TurnBasedMatch::from_payload(match_snapshot),
        },
        LocalPlayerEventPayload::ModifiedSavedGame { player, saved_game } => {
            LocalPlayerEvent::ModifiedSavedGame { player, saved_game }
        }
        LocalPlayerEventPayload::ConflictingSavedGames {
            player,
            saved_games,
        } => LocalPlayerEvent::ConflictingSavedGames {
            player,
            saved_games,
        },
        LocalPlayerEventPayload::WantsToPlayGameActivity { player, activity } => {
            LocalPlayerEvent::WantsToPlayGameActivity {
                player,
                activity: GameActivitySnapshot::from_payload(activity),
            }
        }
    })
}

unsafe fn parse_event_payload(payload_json: *const c_char) -> Option<LocalPlayerEventPayload> {
    if payload_json.is_null() {
        return None;
    }

    CStr::from_ptr(payload_json)
        .to_str()
        .ok()
        .and_then(|json| serde_json::from_str::<LocalPlayerEventPayload>(json).ok())
}
