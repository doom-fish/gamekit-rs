//! Unit tests for `async_api` module.
//!
//! These tests verify the Future newtypes compile and that the `poll` logic
//! correctly maps `Ok` / `Err` results to the expected Rust types.  They use
//! `pollster::block_on` as the executor and drive futures with a pre-loaded
//! `AsyncCompletion` so no actual `GameKit` calls are made.

#![cfg(feature = "async")]

use doom_fish_utils::completion::AsyncCompletion;

// ---------------------------------------------------------------------------
// Helper: manufacture a completion-resolved future pre-seeded before polling.
// ---------------------------------------------------------------------------

fn ready_ok<T: Send + 'static>(value: T) -> doom_fish_utils::completion::AsyncCompletionFuture<T> {
    let (fut, ctx) = AsyncCompletion::create();
    unsafe { AsyncCompletion::complete_ok(ctx, value) };
    fut
}

fn ready_err<T: Send + 'static>(
    msg: &str,
) -> doom_fish_utils::completion::AsyncCompletionFuture<T> {
    let (fut, ctx) = AsyncCompletion::create();
    unsafe { AsyncCompletion::<T>::complete_err(ctx, msg.to_owned()) };
    fut
}

// ---------------------------------------------------------------------------
// FriendsAuthorizationFuture — happy path and error path
// ---------------------------------------------------------------------------

#[test]
fn friends_authorization_future_ok() {
    use gamekit::error::GameKitError;
    use gamekit::local_player::FriendsAuthorizationStatus;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::Poll;

    struct TestFut(doom_fish_utils::completion::AsyncCompletionFuture<String>);
    impl Future for TestFut {
        type Output = Result<FriendsAuthorizationStatus, GameKitError>;
        fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
            Pin::new(&mut self.0).poll(cx).map(|r| {
                r.map_err(GameKitError::Unknown).and_then(|s| {
                    let raw: i32 = s.parse().map_err(|_| {
                        GameKitError::Unknown(format!("bad auth status: {s}"))
                    })?;
                    Ok(match raw {
                        1 => FriendsAuthorizationStatus::Restricted,
                        2 => FriendsAuthorizationStatus::Denied,
                        3 => FriendsAuthorizationStatus::Authorized,
                        _ => FriendsAuthorizationStatus::NotDetermined,
                    })
                })
            })
        }
    }

    // Raw value `3` = Authorized.
    let inner = ready_ok::<String>("3".to_owned());

    assert_eq!(
        pollster::block_on(TestFut(inner)).unwrap(),
        FriendsAuthorizationStatus::Authorized
    );
}

#[test]
fn friends_authorization_future_err() {
    use gamekit::error::GameKitError;
    use gamekit::local_player::FriendsAuthorizationStatus;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::Poll;

    struct TestFut(doom_fish_utils::completion::AsyncCompletionFuture<String>);
    impl Future for TestFut {
        type Output = Result<FriendsAuthorizationStatus, GameKitError>;
        fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
            Pin::new(&mut self.0)
                .poll(cx)
                .map(|r| r.map_err(GameKitError::Unknown).map(|_| FriendsAuthorizationStatus::NotDetermined))
        }
    }

    let inner = ready_err::<String>("bridge error");

    let err = pollster::block_on(TestFut(inner)).unwrap_err();
    assert!(matches!(err, GameKitError::Unknown(ref s) if s == "bridge error"));
}

// ---------------------------------------------------------------------------
// FetchAllSavedGamesFuture — happy path (SavedGame derives Deserialize)
// ---------------------------------------------------------------------------

#[test]
fn fetch_all_saved_games_future_ok() {
    use gamekit::error::GameKitError;
    use gamekit::save::SavedGame;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::Poll;

    struct TestFut(doom_fish_utils::completion::AsyncCompletionFuture<String>);
    impl Future for TestFut {
        type Output = Result<Vec<SavedGame>, GameKitError>;
        fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
            Pin::new(&mut self.0).poll(cx).map(|r| {
                r.map_err(GameKitError::Unknown).and_then(|json| {
                    serde_json::from_str(&json).map_err(|e| {
                        GameKitError::Unknown(format!("bad saved games JSON: {e}"))
                    })
                })
            })
        }
    }

    let json = r#"[{"name":"slot1","deviceName":"MyMac","modificationDate":"2026-01-01T00:00:00.000Z"}]"#;
    let inner = ready_ok::<String>(json.to_owned());

    let games = pollster::block_on(TestFut(inner)).unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].name.as_deref(), Some("slot1"));
}

#[test]
fn fetch_all_saved_games_future_err() {
    use gamekit::error::GameKitError;
    use gamekit::save::SavedGame;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::Poll;

    struct TestFut(doom_fish_utils::completion::AsyncCompletionFuture<String>);
    impl Future for TestFut {
        type Output = Result<Vec<SavedGame>, GameKitError>;
        fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
            Pin::new(&mut self.0)
                .poll(cx)
                .map(|r| r.map_err(GameKitError::Unknown).map(|_| vec![]))
        }
    }

    let inner = ready_err::<String>("iCloud unavailable");

    assert!(pollster::block_on(TestFut(inner)).is_err());
}

// ---------------------------------------------------------------------------
// ReportAchievementFuture — happy path and error path
// ---------------------------------------------------------------------------

#[test]
fn report_achievement_future_ok() {
    use gamekit::error::GameKitError;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::Poll;

    struct TestFut(doom_fish_utils::completion::AsyncCompletionFuture<String>);
    impl Future for TestFut {
        type Output = Result<(), GameKitError>;
        fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
            Pin::new(&mut self.0)
                .poll(cx)
                .map(|r| r.map(|_| ()).map_err(GameKitError::Unknown))
        }
    }

    let inner = ready_ok::<String>("null".to_owned());

    assert!(pollster::block_on(TestFut(inner)).is_ok());
}

#[test]
fn report_achievement_future_err() {
    use gamekit::error::GameKitError;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::Poll;

    struct TestFut(doom_fish_utils::completion::AsyncCompletionFuture<String>);
    impl Future for TestFut {
        type Output = Result<(), GameKitError>;
        fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
            Pin::new(&mut self.0)
                .poll(cx)
                .map(|r| r.map(|_| ()).map_err(GameKitError::Unknown))
        }
    }

    let inner = ready_err::<String>("not authenticated");

    let err = pollster::block_on(TestFut(inner)).unwrap_err();
    assert!(matches!(err, GameKitError::Unknown(ref s) if s == "not authenticated"));
}

// ---------------------------------------------------------------------------
// Smoke test: all public async types are accessible
// ---------------------------------------------------------------------------

#[test]
fn async_types_are_accessible() {
    let _ = std::mem::size_of::<gamekit::async_api::AuthenticateFuture>();
    let _ = std::mem::size_of::<gamekit::async_api::FriendsAuthorizationFuture>();
    let _ = std::mem::size_of::<gamekit::async_api::FindMatchFuture>();
    let _ = std::mem::size_of::<gamekit::async_api::FindPlayersFuture>();
    let _ = std::mem::size_of::<gamekit::async_api::LoadLeaderboardsFuture>();
    let _ = std::mem::size_of::<gamekit::async_api::LoadEntriesFuture>();
    let _ = std::mem::size_of::<gamekit::async_api::LoadAchievementsFuture>();
    let _ = std::mem::size_of::<gamekit::async_api::ReportAchievementFuture>();
    let _ = std::mem::size_of::<gamekit::async_api::FetchAllSavedGamesFuture>();
    let _ = std::mem::size_of::<gamekit::async_api::LoadSavedGameDataFuture>();
    let _ = std::mem::size_of::<gamekit::async_api::SaveGameFuture>();
    let _ = std::mem::size_of::<gamekit::async_api::AsyncLocalPlayer>();
    let _ = std::mem::size_of::<gamekit::async_api::AsyncMatchmaker>();
    let _ = std::mem::size_of::<gamekit::async_api::AsyncLeaderboard>();
    let _ = std::mem::size_of::<gamekit::async_api::AsyncAchievement>();
    let _ = std::mem::size_of::<gamekit::async_api::AsyncSavedGame>();
}
