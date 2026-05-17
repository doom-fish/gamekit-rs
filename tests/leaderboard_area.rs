mod common;

use common::sample_player;

#[test]
fn leaderboard_area_tracks_recurring_snapshots_and_entries() {
    let leaderboard = gamekit::Leaderboard {
        base_leaderboard_id: "com.example.high-score".to_owned(),
        title: Some("High Score".to_owned()),
        group_identifier: Some("main".to_owned()),
        leaderboard_type: gamekit::LeaderboardType::Recurring,
        start_date: Some("2026-01-01T00:00:00Z".to_owned()),
        next_start_date: Some("2026-01-08T00:00:00Z".to_owned()),
        duration_seconds: Some(604_800.0),
    };
    let local_entry = gamekit::LeaderboardEntry {
        rank: 1,
        score: 9_999,
        formatted_score: "9,999".to_owned(),
        context: 7,
        date: "2026-01-01T00:00:00Z".to_owned(),
        player: sample_player(),
    };
    let mut rival = sample_player();
    rival.game_player_id = "G:player-2".to_owned();
    rival.team_player_id = "T:player-2".to_owned();
    rival.alias = "challenger".to_owned();
    rival.display_name = "The Challenger".to_owned();
    rival.player_id = Some("legacy-player-2".to_owned());
    let rival_entry = gamekit::LeaderboardEntry {
        rank: 2,
        score: 8_750,
        formatted_score: "8,750".to_owned(),
        context: 5,
        date: "2026-01-01T00:05:00Z".to_owned(),
        player: rival,
    };
    let result = gamekit::LoadEntriesResult {
        local_player_entry: Some(local_entry.clone()),
        entries: vec![local_entry.clone(), rival_entry.clone()],
        total_player_count: 2,
    };

    assert_eq!(
        leaderboard.leaderboard_type,
        gamekit::LeaderboardType::Recurring
    );
    assert_eq!(
        leaderboard.start_date.as_deref(),
        Some("2026-01-01T00:00:00Z")
    );
    assert_eq!(result.local_player_entry.as_ref(), Some(&local_entry));
    assert_eq!(result.entries[1], rival_entry);
    assert_eq!(result.total_player_count, 2);
    assert_eq!(
        gamekit::PlayerScope::FriendsOnly,
        gamekit::PlayerScope::FriendsOnly
    );
    assert_eq!(gamekit::TimeScope::Week, gamekit::TimeScope::Week);
}
