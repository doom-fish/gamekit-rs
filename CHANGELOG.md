# Changelog

## [0.1.0] - 2024-01-01

### Added
- `LocalPlayer::local()` — snapshot of the authenticated local player
- `AuthObserver` — authenticate-handler guard with Rust closure callback
- `Leaderboard::load()`, `submit_score()`, `load_entries()`
- `Achievement::report()`, `Achievement::report_all()`
- `Matchmaker::shared()`, `find_match()`, `cancel()`
- `Match` with `connected_players()`, `send_data()`, `set_delegate()`
- `Player` struct with `game_player_id`, `alias`, `display_name`
