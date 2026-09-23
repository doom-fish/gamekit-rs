mod common;

use common::sample_local_player;
use gamekit::{
    AuthEvent, BasePlayer, FriendsAuthorizationStatus, GameKitError, IdentityVerificationSignature,
    LocalPlayer, PLAYER_AUTHENTICATION_DID_CHANGE_NOTIFICATION_NAME,
};
use serde_json::json;

#[test]
fn local_player_area_exposes_identity_and_auth_views() {
    let local = sample_local_player();
    let from_ref: BasePlayer = (&local).into();
    let from_owned: BasePlayer = local.clone().into();
    let auth_event: AuthEvent = serde_json::from_value(json!({
        "hasViewController": false,
        "error": null
    }))
    .expect("deserialize auth event");
    let signature: IdentityVerificationSignature = serde_json::from_value(json!({
        "publicKeyUrl": "https://example.com/key",
        "signatureBase64": "c2lnbmF0dXJl",
        "saltBase64": "c2FsdA==",
        "timestamp": 1_717_171_717_u64
    }))
    .expect("deserialize identity verification signature");

    assert!(local.is_authenticated);
    assert_eq!(from_ref, from_owned);
    assert_eq!(from_ref.display_name.as_deref(), Some("Doom Fish"));
    assert!(!auth_event.has_view_controller);
    assert!(auth_event.error.is_none());
    assert_eq!(signature.public_key_url, "https://example.com/key");
    assert_eq!(signature.timestamp, 1_717_171_717);
    assert_eq!(
        FriendsAuthorizationStatus::Authorized,
        FriendsAuthorizationStatus::Authorized
    );
    assert_eq!(
        PLAYER_AUTHENTICATION_DID_CHANGE_NOTIFICATION_NAME,
        "GKPlayerAuthenticationDidChangeNotificationName"
    );
}

#[test]
fn friend_apis_without_a_bundle_identifier_return_errors() {
    let status = LocalPlayer::load_friends_authorization_status();
    assert!(
        matches!(status, Err(GameKitError::Unavailable(_))),
        "friends authorization: {status:?}"
    );
    let friends = LocalPlayer::load_friends();
    assert!(
        matches!(friends, Err(GameKitError::Unavailable(_))),
        "friends: {friends:?}"
    );
    let identified = LocalPlayer::load_friends_identified_by(&["G:1"]);
    assert!(
        matches!(identified, Err(GameKitError::Unavailable(_))),
        "friends by identifier: {identified:?}"
    );
}
