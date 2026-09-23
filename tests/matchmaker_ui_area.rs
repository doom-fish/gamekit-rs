use std::sync::Arc;

use gamekit::{
    DialogController, GameKitError, MatchRequest, MatchmakerViewController, TurnBasedMatchRequest,
    TurnBasedMatchmakerViewController,
};

trait AmbiguousIfSend<A> {
    fn check() {}
}

impl<T: ?Sized> AmbiguousIfSend<()> for T {}
impl<T: ?Sized + Send> AmbiguousIfSend<u8> for T {}

#[test]
fn view_controllers_and_delegate_guards_are_not_send() {
    <MatchmakerViewController as AmbiguousIfSend<_>>::check();
    <TurnBasedMatchmakerViewController as AmbiguousIfSend<_>>::check();
    <gamekit::MatchmakerViewControllerDelegate as AmbiguousIfSend<_>>::check();
    <gamekit::TurnBasedMatchmakerViewControllerDelegate as AmbiguousIfSend<_>>::check();
    <gamekit::GameCenterControllerDelegate as AmbiguousIfSend<_>>::check();
}

#[test]
fn view_controllers_refuse_creation_off_the_main_thread() {
    let real_time = MatchmakerViewController::new(&MatchRequest::default());
    assert!(
        matches!(real_time, Err(GameKitError::WrongThread(_))),
        "{:?}",
        real_time.err()
    );
    let turn_based = TurnBasedMatchmakerViewController::new(&TurnBasedMatchRequest::default());
    assert!(
        matches!(turn_based, Err(GameKitError::WrongThread(_))),
        "{:?}",
        turn_based.err()
    );
}

#[test]
fn dialog_calls_refuse_to_run_off_the_main_thread() {
    let dismissed = DialogController::shared().dismiss();
    assert!(
        matches!(dismissed, Err(GameKitError::WrongThread(_))),
        "{dismissed:?}"
    );

    let witness = Arc::new(());
    let captured = Arc::clone(&witness);
    let presented = DialogController::shared().present_game_center_dashboard(move || {
        let _ = &captured;
    });
    assert!(
        matches!(presented, Err(GameKitError::WrongThread(_))),
        "{:?}",
        presented.err()
    );
    assert_eq!(Arc::strong_count(&witness), 1);
}
