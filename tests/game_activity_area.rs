mod common;

use std::collections::BTreeMap;

use common::sample_player;
use gamekit::{
    Achievement, GameActivityDefinition, GameActivityPlayStyle, GameActivitySnapshot,
    GameActivityState, Score,
};

#[test]
fn game_activity_area_tracks_party_code_properties_and_scores() {
    let definition = GameActivityDefinition {
        identifier: "raid-night".to_owned(),
        group_identifier: Some("weekly".to_owned()),
        title: "Raid Night".to_owned(),
        details: Some("Weekly co-op raid".to_owned()),
        default_properties: BTreeMap::from([
            ("difficulty".to_owned(), "heroic".to_owned()),
            ("role".to_owned(), "tank".to_owned()),
        ]),
        fallback_url: Some("https://example.com/raid".to_owned()),
        supports_party_code: true,
        max_players: Some(6),
        min_players: Some(2),
        supports_unlimited_players: false,
        play_style: GameActivityPlayStyle::Synchronous,
        release_state: "released".to_owned(),
    };
    let snapshot = GameActivitySnapshot {
        identifier: "activity-1".to_owned(),
        activity_definition: definition.clone(),
        properties: BTreeMap::from([
            ("difficulty".to_owned(), "heroic".to_owned()),
            ("map".to_owned(), "volcano".to_owned()),
        ]),
        state: GameActivityState::Active,
        party_code: Some("ABCD-EFGH".to_owned()),
        party_url: Some("https://example.com/join/ABCD-EFGH".to_owned()),
        creation_date: "2026-01-01T00:00:00Z".to_owned(),
        start_date: Some("2026-01-01T00:01:00Z".to_owned()),
        last_resume_date: Some("2026-01-01T00:05:00Z".to_owned()),
        end_date: None,
        duration_seconds: 300.0,
        achievements: vec![Achievement {
            identifier: "raid-clear".to_owned(),
            percent_complete: 50.0,
            is_completed: false,
            last_reported_date: Some("2026-01-01T00:10:00Z".to_owned()),
            shows_completion_banner: false,
            player: Some(sample_player()),
        }],
        leaderboard_scores: vec![
            Score::new_local("com.example.raid", 7_500, 3),
            Score::for_player_game_id("com.example.raid", 8_200, 4, "G:player-1"),
        ],
    };

    assert_eq!(snapshot.activity_definition, definition);
    assert_eq!(snapshot.state, GameActivityState::Active);
    assert_eq!(snapshot.party_code.as_deref(), Some("ABCD-EFGH"));
    assert_eq!(
        snapshot.properties.get("map").map(String::as_str),
        Some("volcano")
    );
    assert_eq!(snapshot.achievements.len(), 1);
    assert_eq!(
        snapshot.achievements[0]
            .player
            .as_ref()
            .map(|player| player.alias.as_str()),
        Some("doomfish")
    );
    assert_eq!(snapshot.leaderboard_scores.len(), 2);
    assert_eq!(
        snapshot.leaderboard_scores[1].player_game_id.as_deref(),
        Some("G:player-1")
    );
}
