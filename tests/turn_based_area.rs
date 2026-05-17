mod common;

use common::sample_player;
use gamekit::{
    TurnBasedExchange, TurnBasedExchangeReply, TurnBasedExchangeStatus, TurnBasedMatch,
    TurnBasedMatchOutcome, TurnBasedMatchRequest, TurnBasedMatchStatus, TurnBasedParticipant,
    TurnBasedParticipantStatus, EXCHANGE_TIMEOUT_NONE, TURN_TIMEOUT_DEFAULT,
};

#[test]
fn turn_based_area_preserves_request_participant_and_exchange_state() {
    let request = TurnBasedMatchRequest {
        max_players: 4,
        recipient_ids: vec!["G:player-2".to_owned()],
        ..TurnBasedMatchRequest::default()
    };
    let participant = TurnBasedParticipant {
        index: 0,
        player: Some(sample_player()),
        last_turn_date: Some("2026-01-01T00:00:00Z".to_owned()),
        status: TurnBasedParticipantStatus::Active,
        match_outcome: TurnBasedMatchOutcome::Won,
        timeout_date: Some("2026-01-02T00:00:00Z".to_owned()),
    };
    let reply = TurnBasedExchangeReply {
        recipient_index: Some(0),
        message: Some("Roger that".to_owned()),
        data_len: 4,
        reply_date: Some("2026-01-01T00:00:05Z".to_owned()),
    };
    let exchange = TurnBasedExchange {
        index: 1,
        exchange_id: Some("exchange-1".to_owned()),
        sender_index: Some(0),
        recipient_indices: vec![1],
        status: TurnBasedExchangeStatus::Resolved,
        message: Some("Your move".to_owned()),
        data_len: 8,
        send_date: Some("2026-01-01T00:00:01Z".to_owned()),
        timeout_date: None,
        completion_date: Some("2026-01-01T00:10:00Z".to_owned()),
        replies: vec![reply.clone()],
    };
    let snapshot = TurnBasedMatch {
        match_id: "turn-based-1".to_owned(),
        creation_date: Some("2026-01-01T00:00:00Z".to_owned()),
        participants: vec![participant.clone()],
        status: TurnBasedMatchStatus::Open,
        current_participant_index: Some(0),
        message: Some("Take your turn".to_owned()),
        match_data_len: 16,
        match_data_maximum_size: 1_024,
        exchanges: vec![exchange.clone()],
        active_exchange_indices: vec![],
        completed_exchange_indices: vec![1],
        exchange_data_maximum_size: 512,
        exchange_max_initiated_exchanges_per_player: 1,
    };

    assert_eq!(request.min_players, 2);
    assert_eq!(request.max_players, 4);
    assert_eq!(request.recipient_ids, vec!["G:player-2".to_owned()]);
    assert_eq!(snapshot.participants[0], participant);
    assert_eq!(snapshot.exchanges[0], exchange);
    assert_eq!(snapshot.completed_exchange_indices, vec![1]);
    assert_eq!(
        TurnBasedMatchOutcome::Custom(99),
        TurnBasedMatchOutcome::Custom(99)
    );
    assert!((TURN_TIMEOUT_DEFAULT - 604_800.0).abs() < f64::EPSILON);
    assert!(EXCHANGE_TIMEOUT_NONE.abs() < f64::EPSILON);
}
