use std::collections::BTreeMap;
use std::mem::size_of;

fn sample_player() -> gamekit::Player {
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

#[test]
fn local_player_area_constructs_snapshot() {
    let player = sample_player();
    let snapshot = gamekit::LocalPlayer {
        is_authenticated: true,
        is_underage: false,
        is_multiplayer_gaming_restricted: false,
        is_personalized_communication_restricted: false,
        is_presenting_friend_request_view_controller: false,
        player: player.clone(),
    };

    assert!(snapshot.is_authenticated);
    assert_eq!(snapshot.player, player);
}

#[test]
fn player_area_round_trips_json() {
    let player = sample_player();
    let json = serde_json::to_string(&player).expect("serialize player");
    let decoded: gamekit::Player = serde_json::from_str(&json).expect("deserialize player");

    assert_eq!(decoded, player);
}

#[test]
fn leaderboard_area_constructs_snapshot() {
    let leaderboard = gamekit::Leaderboard {
        base_leaderboard_id: "com.example.high-score".to_owned(),
        title: Some("High Score".to_owned()),
        group_identifier: Some("main".to_owned()),
        leaderboard_type: gamekit::LeaderboardType::Recurring,
        start_date: Some("2026-01-01T00:00:00Z".to_owned()),
        next_start_date: Some("2026-01-08T00:00:00Z".to_owned()),
        duration_seconds: Some(604_800.0),
    };

    assert_eq!(leaderboard.base_leaderboard_id, "com.example.high-score");
    assert_eq!(
        leaderboard.leaderboard_type,
        gamekit::LeaderboardType::Recurring
    );
}

#[test]
fn leaderboard_entry_area_constructs_result() {
    let entry = gamekit::LeaderboardEntry {
        rank: 1,
        score: 9_999,
        formatted_score: "9,999".to_owned(),
        context: 7,
        date: "2026-01-01T00:00:00Z".to_owned(),
        player: sample_player(),
    };
    let result = gamekit::LoadEntriesResult {
        local_player_entry: Some(entry.clone()),
        entries: vec![entry.clone()],
        total_player_count: 1,
    };

    assert_eq!(result.local_player_entry, Some(entry));
    assert_eq!(result.total_player_count, 1);
}

#[test]
fn achievement_area_constructs_models() {
    let achievement = gamekit::Achievement {
        identifier: "first-win".to_owned(),
        percent_complete: 100.0,
        is_completed: true,
        last_reported_date: Some("2026-01-01T00:00:00Z".to_owned()),
        shows_completion_banner: true,
        player: Some(sample_player()),
    };
    let description = gamekit::AchievementDescription {
        identifier: Some("first-win".to_owned()),
        group_identifier: Some("progression".to_owned()),
        title: Some("First Win".to_owned()),
        achieved_description: Some("You won your first match.".to_owned()),
        unachieved_description: Some("Win one match.".to_owned()),
        maximum_points: 10,
        is_hidden: false,
        is_replayable: false,
    };

    assert!(achievement.is_completed);
    assert_eq!(description.maximum_points, 10);
}

#[test]
fn match_area_constructs_event() {
    let player = sample_player();
    let event = gamekit::MatchEvent::ConnectionStateChanged {
        player: player.clone(),
        state: gamekit::ConnectionState::Connected,
    };

    match event {
        gamekit::MatchEvent::ConnectionStateChanged {
            player: event_player,
            state,
        } => {
            assert_eq!(event_player, player);
            assert_eq!(state, gamekit::ConnectionState::Connected);
        }
        gamekit::MatchEvent::ReceivedData { .. } | gamekit::MatchEvent::Failed { .. } => {
            panic!("unexpected match event variant")
        }
    }

    assert_eq!(
        gamekit::SendDataMode::Reliable,
        gamekit::SendDataMode::Reliable
    );
}

#[test]
fn turn_based_area_constructs_models() {
    let request = gamekit::TurnBasedMatchRequest::default();
    let participant = gamekit::TurnBasedParticipant {
        index: 0,
        player: Some(sample_player()),
        last_turn_date: Some("2026-01-01T00:00:00Z".to_owned()),
        status: gamekit::TurnBasedParticipantStatus::Active,
        match_outcome: gamekit::TurnBasedMatchOutcome::None,
        timeout_date: None,
    };
    let reply = gamekit::TurnBasedExchangeReply {
        recipient_index: Some(0),
        message: Some("Roger that".to_owned()),
        data_len: 4,
        reply_date: Some("2026-01-01T00:00:05Z".to_owned()),
    };
    let exchange = gamekit::TurnBasedExchange {
        index: 0,
        exchange_id: Some("exchange-1".to_owned()),
        sender_index: Some(0),
        recipient_indices: vec![1],
        status: gamekit::TurnBasedExchangeStatus::Active,
        message: Some("Your move".to_owned()),
        data_len: 8,
        send_date: Some("2026-01-01T00:00:01Z".to_owned()),
        timeout_date: None,
        completion_date: None,
        replies: vec![reply],
    };
    let match_snapshot = gamekit::TurnBasedMatch {
        match_id: "turn-based-1".to_owned(),
        creation_date: Some("2026-01-01T00:00:00Z".to_owned()),
        participants: vec![participant],
        status: gamekit::TurnBasedMatchStatus::Open,
        current_participant_index: Some(0),
        message: Some("Take your turn".to_owned()),
        match_data_len: 16,
        match_data_maximum_size: 1024,
        exchanges: vec![exchange],
        active_exchange_indices: vec![0],
        completed_exchange_indices: vec![],
        exchange_data_maximum_size: 512,
        exchange_max_initiated_exchanges_per_player: 1,
    };

    assert_eq!(request.min_players, 2);
    assert_eq!(match_snapshot.status, gamekit::TurnBasedMatchStatus::Open);
    assert_eq!(match_snapshot.active_exchange_indices, vec![0]);
}

#[test]
fn real_time_area_uses_request_defaults() {
    let request = gamekit::MatchRequest::default();

    assert_eq!(request.min_players, 2);
    assert_eq!(request.max_players, 4);
    assert!(request.invite_message.is_none());
    assert_eq!(size_of::<gamekit::Matchmaker>(), 0);
    assert_eq!(gamekit::MatchType::Hosted, gamekit::MatchType::Hosted);
}

#[test]
fn notification_area_is_zero_sized() {
    assert_eq!(size_of::<gamekit::NotificationBanner>(), 0);
    let _ = gamekit::NotificationBanner;
}

#[test]
fn access_point_area_handles_snapshot_model() {
    let snapshot = gamekit::AccessPointSnapshot {
        is_active: true,
        is_visible: true,
        is_presenting_game_center: false,
        location: gamekit::AccessPointLocation::TopTrailing,
        frame: gamekit::AccessPointFrame {
            x: 10.0,
            y: 20.0,
            width: 44.0,
            height: 44.0,
        },
    };

    assert!(snapshot.is_visible);
    assert_eq!(snapshot.location, gamekit::AccessPointLocation::TopTrailing);
    assert_eq!(size_of::<gamekit::AccessPoint>(), 0);
    let _ = gamekit::AccessPointState::Dashboard;
}

#[test]
fn challenge_definition_area_constructs_models() {
    let definition = gamekit::ChallengeDefinition {
        identifier: "challenge-1".to_owned(),
        group_identifier: Some("season-1".to_owned()),
        title: "Win Streak".to_owned(),
        details: Some("Win three matches in a row.".to_owned()),
        duration_options: vec![gamekit::ChallengeDurationOption {
            year: None,
            month: None,
            week_of_year: Some(1),
            day: None,
            hour: Some(1),
            minute: None,
            second: None,
        }],
        is_repeatable: true,
        leaderboard_id: Some("com.example.high-score".to_owned()),
        release_state: "released".to_owned(),
    };

    assert_eq!(definition.duration_options.len(), 1);
    assert!(definition.is_repeatable);
}

#[test]
fn score_area_builders_round_trip_json() {
    let local = gamekit::Score::new_local("com.example.high-score", 500, 42);
    let targeted =
        gamekit::Score::for_player_game_id("com.example.high-score", 1_000, 7, "G:player-1");
    let json = serde_json::to_string(&targeted).expect("serialize score");
    let decoded: gamekit::Score = serde_json::from_str(&json).expect("deserialize score");

    assert!(local.player_game_id.is_none());
    assert_eq!(decoded, targeted);
}

#[test]
fn save_area_round_trips_json() {
    let saved_game = gamekit::SavedGame {
        name: Some("autosave".to_owned()),
        device_name: Some("MacBook Pro".to_owned()),
        modification_date: Some("2026-01-01T00:00:00Z".to_owned()),
    };
    let json = serde_json::to_string(&saved_game).expect("serialize saved game");
    let decoded: gamekit::SavedGame = serde_json::from_str(&json).expect("deserialize saved game");

    assert_eq!(decoded, saved_game);
}

#[test]
fn game_activity_area_constructs_models() {
    let definition = gamekit::GameActivityDefinition {
        identifier: "raid-night".to_owned(),
        group_identifier: Some("weekly".to_owned()),
        title: "Raid Night".to_owned(),
        details: Some("Weekly co-op raid".to_owned()),
        default_properties: BTreeMap::from([("difficulty".to_owned(), "heroic".to_owned())]),
        fallback_url: Some("https://example.com/raid".to_owned()),
        supports_party_code: true,
        max_players: Some(6),
        min_players: Some(2),
        supports_unlimited_players: false,
        play_style: gamekit::GameActivityPlayStyle::Synchronous,
        release_state: "released".to_owned(),
    };
    let achievement = gamekit::Achievement {
        identifier: "raid-clear".to_owned(),
        percent_complete: 50.0,
        is_completed: false,
        last_reported_date: Some("2026-01-01T00:10:00Z".to_owned()),
        shows_completion_banner: false,
        player: Some(sample_player()),
    };
    let snapshot = gamekit::GameActivitySnapshot {
        identifier: "activity-1".to_owned(),
        activity_definition: definition.clone(),
        properties: BTreeMap::from([("map".to_owned(), "volcano".to_owned())]),
        state: gamekit::GameActivityState::Active,
        party_code: Some("ABCD-EFGH".to_owned()),
        party_url: Some("https://example.com/join/ABCD-EFGH".to_owned()),
        creation_date: "2026-01-01T00:00:00Z".to_owned(),
        start_date: Some("2026-01-01T00:01:00Z".to_owned()),
        last_resume_date: Some("2026-01-01T00:05:00Z".to_owned()),
        end_date: None,
        duration_seconds: 240.0,
        achievements: vec![achievement],
        leaderboard_scores: vec![gamekit::Score::new_local("com.example.raid", 7_500, 3)],
    };

    assert_eq!(
        definition.play_style,
        gamekit::GameActivityPlayStyle::Synchronous
    );
    assert_eq!(snapshot.state, gamekit::GameActivityState::Active);
    assert_eq!(snapshot.leaderboard_scores.len(), 1);
}

#[test]
fn local_player_listener_area_constructs_events() {
    let saved_game = gamekit::SavedGame {
        name: Some("autosave".to_owned()),
        device_name: Some("MacBook Pro".to_owned()),
        modification_date: Some("2026-01-01T00:00:00Z".to_owned()),
    };
    let definition = gamekit::GameActivityDefinition {
        identifier: "raid-night".to_owned(),
        group_identifier: None,
        title: "Raid Night".to_owned(),
        details: None,
        default_properties: BTreeMap::new(),
        fallback_url: None,
        supports_party_code: false,
        max_players: None,
        min_players: Some(2),
        supports_unlimited_players: false,
        play_style: gamekit::GameActivityPlayStyle::Unspecified,
        release_state: "released".to_owned(),
    };
    let activity = gamekit::GameActivitySnapshot {
        identifier: "activity-1".to_owned(),
        activity_definition: definition,
        properties: BTreeMap::new(),
        state: gamekit::GameActivityState::Initialized,
        party_code: None,
        party_url: None,
        creation_date: "2026-01-01T00:00:00Z".to_owned(),
        start_date: None,
        last_resume_date: None,
        end_date: None,
        duration_seconds: 0.0,
        achievements: vec![],
        leaderboard_scores: vec![],
    };
    let event = gamekit::LocalPlayerEvent::WantsToPlayGameActivity {
        player: sample_player(),
        activity,
    };
    let saved_event = gamekit::LocalPlayerEvent::ModifiedSavedGame {
        player: sample_player(),
        saved_game,
    };

    match event {
        gamekit::LocalPlayerEvent::WantsToPlayGameActivity { player, activity } => {
            assert_eq!(player.alias, "doomfish");
            assert_eq!(activity.state, gamekit::GameActivityState::Initialized);
        }
        _ => panic!("unexpected local-player event variant"),
    }

    match saved_event {
        gamekit::LocalPlayerEvent::ModifiedSavedGame { saved_game, .. } => {
            assert_eq!(saved_game.name.as_deref(), Some("autosave"));
        }
        _ => panic!("unexpected saved-game event variant"),
    }
}

#[test]
fn matchmaking_ui_area_references_types() {
    assert_eq!(size_of::<gamekit::DialogController>(), 0);
    assert_eq!(size_of::<gamekit::Invite>(), size_of::<usize>());
    assert_eq!(
        size_of::<gamekit::MatchmakerViewController>(),
        size_of::<usize>()
    );
    assert_eq!(
        size_of::<gamekit::TurnBasedMatchmakerViewController>(),
        size_of::<usize>()
    );
    assert_eq!(
        gamekit::MatchmakingMode::InviteOnly,
        gamekit::MatchmakingMode::InviteOnly
    );
}
