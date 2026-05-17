mod common;

use common::sample_player;

#[test]
fn achievement_area_preserves_completion_and_visibility_metadata() {
    let achievement = gamekit::Achievement {
        identifier: "first-win".to_owned(),
        percent_complete: 100.0,
        is_completed: true,
        last_reported_date: Some("2026-01-01T00:00:00Z".to_owned()),
        shows_completion_banner: true,
        player: Some(sample_player()),
    };
    let replayable = gamekit::Achievement {
        identifier: "daily-win".to_owned(),
        percent_complete: 50.0,
        is_completed: false,
        last_reported_date: None,
        shows_completion_banner: false,
        player: None,
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

    assert_eq!(achievement.clone(), achievement);
    assert!(achievement.is_completed);
    assert_eq!(
        achievement
            .player
            .as_ref()
            .map(|player| player.alias.as_str()),
        Some("doomfish")
    );
    assert!(!replayable.is_completed);
    assert!(replayable.player.is_none());
    assert_eq!(description.identifier.as_deref(), Some("first-win"));
    assert_eq!(description.maximum_points, 10);
    assert!(!description.is_hidden);
    assert!(!description.is_replayable);
}
