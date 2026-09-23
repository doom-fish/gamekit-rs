use std::sync::Arc;

use gamekit::{
    GameKitError, MatchRequest, MatchmakerViewController, Player, TurnBasedMatchRequest,
    TurnBasedMatchmakerViewController,
};

fn unknown_player() -> Player {
    Player {
        game_player_id: "G:unknown".to_owned(),
        team_player_id: "T:unknown".to_owned(),
        alias: String::new(),
        display_name: String::new(),
        player_id: None,
        guest_identifier: None,
        is_invitable: false,
        scoped_ids_are_persistent: false,
    }
}

fn recipients_are_rejected_on_the_main_thread() {
    let real_time = MatchmakerViewController::new(&MatchRequest {
        recipient_ids: vec!["G:recipient".to_owned()],
        ..MatchRequest::default()
    });
    assert!(
        matches!(real_time, Err(GameKitError::Unavailable(_))),
        "{:?}",
        real_time.err()
    );
    let turn_based = TurnBasedMatchmakerViewController::new(&TurnBasedMatchRequest {
        recipient_ids: vec!["G:recipient".to_owned()],
        ..TurnBasedMatchRequest::default()
    });
    assert!(
        matches!(turn_based, Err(GameKitError::Unavailable(_))),
        "{:?}",
        turn_based.err()
    );
    println!("test recipients_are_rejected_on_the_main_thread ... ok");
}

fn delegate_contexts_are_released_with_their_guards() {
    let witness = Arc::new(());

    let controller = MatchmakerViewController::new(&MatchRequest::default())
        .expect("matchmaking view controller on the main thread");
    let captured = Arc::clone(&witness);
    let delegate = controller.set_delegate(move |_| {
        let _ = &captured;
    });
    let hosted = controller.set_hosted_player_connected(&unknown_player(), true);
    assert!(
        matches!(hosted, Err(GameKitError::NotFound(_))),
        "{hosted:?}"
    );
    let replaced = controller.set_delegate(|_| {});
    drop(delegate);
    assert_eq!(Arc::strong_count(&witness), 1);
    drop(replaced);
    drop(controller);

    let controller = TurnBasedMatchmakerViewController::new(&TurnBasedMatchRequest::default())
        .expect("turn-based matchmaking view controller on the main thread");
    let captured = Arc::clone(&witness);
    let delegate = controller.set_delegate(move |_| {
        let _ = &captured;
    });
    assert_eq!(Arc::strong_count(&witness), 2);
    drop(delegate);
    assert_eq!(Arc::strong_count(&witness), 1);
    drop(controller);
    println!("test delegate_contexts_are_released_with_their_guards ... ok");
}

fn main() {
    recipients_are_rejected_on_the_main_thread();
    delegate_contexts_are_released_with_their_guards();
}
