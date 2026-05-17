#![allow(dead_code)]

pub fn sample_player() -> gamekit::Player {
    gamekit::Player {
        game_player_id: "G:player-1".to_owned(),
        team_player_id: "T:player-1".to_owned(),
        alias: "doomfish".to_owned(),
        display_name: "Doom Fish".to_owned(),
        player_id: Some("legacy-player-1".to_owned()),
        guest_identifier: None,
        is_invitable: true,
        scoped_ids_are_persistent: true,
    }
}

pub fn sample_local_player() -> gamekit::LocalPlayer {
    gamekit::LocalPlayer {
        is_authenticated: true,
        is_underage: false,
        is_multiplayer_gaming_restricted: false,
        is_personalized_communication_restricted: false,
        is_presenting_friend_request_view_controller: false,
        player: sample_player(),
    }
}
