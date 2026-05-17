mod common;

use common::sample_player;
use gamekit::r#match::GameKitFrameworkErrorData;
use gamekit::{ConnectionState, MatchEvent, SendDataMode, ERROR_DOMAIN};
use serde_json::json;

#[test]
fn match_area_covers_data_state_and_failure_events() {
    let player = sample_player();
    let payload = vec![1_u8, 2, 3, 4];

    let data_event = MatchEvent::ReceivedData {
        player: player.clone(),
        data: payload.clone(),
    };
    match data_event {
        MatchEvent::ReceivedData {
            player: event_player,
            data,
        } => {
            assert_eq!(event_player, player);
            assert_eq!(data, payload);
        }
        MatchEvent::ConnectionStateChanged { .. } | MatchEvent::Failed { .. } => {
            panic!("unexpected event variant")
        }
    }

    let state_event = MatchEvent::ConnectionStateChanged {
        player: player.clone(),
        state: ConnectionState::Disconnected,
    };
    match state_event {
        MatchEvent::ConnectionStateChanged {
            player: event_player,
            state,
        } => {
            assert_eq!(event_player.team_player_id, player.team_player_id);
            assert_eq!(state, ConnectionState::Disconnected);
        }
        MatchEvent::ReceivedData { .. } | MatchEvent::Failed { .. } => {
            panic!("unexpected event variant")
        }
    }

    let failure_error: GameKitFrameworkErrorData = serde_json::from_value(json!({
        "kind": "framework",
        "domain": ERROR_DOMAIN,
        "code": 6,
        "localizedDescription": "connection lost"
    }))
    .expect("deserialize match failure");
    let failure_event = MatchEvent::Failed {
        error: Some(failure_error),
    };
    match failure_event {
        MatchEvent::Failed { error: Some(error) } => {
            assert_eq!(error.domain, ERROR_DOMAIN);
            assert_eq!(error.code, 6);
            assert_eq!(error.localized_description, "connection lost");
        }
        MatchEvent::ReceivedData { .. }
        | MatchEvent::ConnectionStateChanged { .. }
        | MatchEvent::Failed { error: None } => panic!("missing match error payload"),
    }

    assert_ne!(SendDataMode::Reliable, SendDataMode::Unreliable);
}
