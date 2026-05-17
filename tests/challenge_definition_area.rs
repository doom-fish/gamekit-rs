use gamekit::ChallengeDefinition;
use serde_json::json;

#[test]
fn challenge_definition_area_deserializes_duration_options() {
    let definition: ChallengeDefinition = serde_json::from_value(json!({
        "identifier": "challenge-1",
        "groupIdentifier": "season-1",
        "title": "Perfect Lap",
        "details": "Finish with no collisions.",
        "durationOptions": [
            {
                "weekOfYear": 24,
                "hour": 6,
                "minute": 30
            },
            {
                "day": 1,
                "second": 45
            }
        ],
        "isRepeatable": true,
        "leaderboardId": "com.example.laps",
        "releaseState": "released"
    }))
    .expect("deserialize challenge definition");

    assert_eq!(definition.identifier, "challenge-1");
    assert_eq!(definition.group_identifier.as_deref(), Some("season-1"));
    assert_eq!(definition.duration_options.len(), 2);
    assert_eq!(definition.duration_options[0].week_of_year, Some(24));
    assert_eq!(definition.duration_options[0].minute, Some(30));
    assert_eq!(definition.duration_options[1].day, Some(1));
    assert_eq!(definition.duration_options[1].second, Some(45));
    assert!(definition.is_repeatable);
    assert_eq!(
        definition.leaderboard_id.as_deref(),
        Some("com.example.laps")
    );
}
