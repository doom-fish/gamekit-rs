use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use gamekit::{AccessPoint, AccessPointState, AccessPointTrigger, GameKitError, Player};

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

#[test]
fn triggers_return_without_waiting_for_game_center() {
    let started = Instant::now();
    AccessPoint::shared().trigger().expect("default trigger");
    AccessPoint::shared()
        .trigger_state(AccessPointState::Dashboard)
        .expect("dashboard trigger");
    AccessPoint::shared()
        .trigger_target(
            AccessPointTrigger::Leaderboard {
                id: "leaderboard",
                player_scope: gamekit::PlayerScope::Global,
                time_scope: gamekit::TimeScope::AllTime,
            },
            |_| {},
        )
        .or_else(|error| match error {
            GameKitError::Unavailable(_) => Ok(()),
            other => Err(other),
        })
        .expect("leaderboard trigger");
    for target in [
        AccessPointTrigger::PlayTogether,
        AccessPointTrigger::Challenges,
        AccessPointTrigger::ChallengeDefinition("challenge"),
        AccessPointTrigger::GameActivityDefinition("activity"),
        AccessPointTrigger::Friending,
        AccessPointTrigger::Arcade,
    ] {
        let result = AccessPoint::shared().trigger_target(target, |_| {});
        assert!(
            matches!(result, Ok(()) | Err(GameKitError::Unavailable(_))),
            "{result:?}"
        );
    }
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn completion_is_dropped_unrun_when_the_trigger_fails() {
    let ran = Arc::new(AtomicBool::new(false));

    let flag = Arc::clone(&ran);
    let invalid = AccessPoint::shared().trigger_target(
        AccessPointTrigger::Achievement("bad\0identifier"),
        move |_| flag.store(true, Ordering::SeqCst),
    );
    assert!(invalid.is_err());
    assert_eq!(Arc::strong_count(&ran), 1);

    let flag = Arc::clone(&ran);
    let player = unknown_player();
    let unresolvable = AccessPoint::shared()
        .trigger_target(AccessPointTrigger::Player(&player), move |_| {
            flag.store(true, Ordering::SeqCst)
        });
    assert!(
        matches!(unresolvable, Err(GameKitError::Unavailable(_))),
        "{unresolvable:?}"
    );
    assert_eq!(Arc::strong_count(&ran), 1);
    assert!(!ran.load(Ordering::SeqCst));
}
