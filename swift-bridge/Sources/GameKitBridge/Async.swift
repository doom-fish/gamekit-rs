import Foundation
import GameKit

// ============================================================================
// Async thunks for gamekit-rs async_api module (Tier 1)
//
// Each thunk takes a C callback of the form:
//   (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void
// where arg1 = JSON result (nil on error), arg2 = error string (nil on success),
// arg3 = opaque Rust context pointer.  The thunk launches a Swift Task and fires
// the callback exactly once, allowing the Rust AsyncCompletion<String> future to
// resolve.  For findMatch, which returns a live GKMatch, the callback uses
//   (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void
// where arg1 is a retained GKMatchBox pointer.
// ============================================================================

// MARK: - GKLocalPlayer

/// Async authenticate: sets `GKLocalPlayer.local.authenticateHandler` and fires
/// the C callback exactly once — on the first non-UI-presenting invocation.
///
/// - If Game Center UI would be required (viewController != nil) the callback
///   fires with an error string `"game_center_ui_required"`.
/// - On headless/CI machines this typically fires immediately with
///   `GKErrorDomain 6` (not authenticated).
@_cdecl("gk_local_player_authenticate_async")
public func gk_local_player_authenticate_async(
    _ cb: @convention(c) (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let lock = NSLock()
    var fired = false
    GKLocalPlayer.local.authenticateHandler = { viewController, error in
        lock.lock()
        guard !fired else { lock.unlock(); return }
        fired = true
        lock.unlock()

        if let error {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        } else if viewController != nil {
            "game_center_ui_required".withCString { cb(nil, $0, ctx) }
        } else {
            if let json = try? gkEncodeJSON(gkLocalPlayerPayload()) {
                json.withCString { cb($0, nil, ctx) }
            } else {
                "failed to encode local player".withCString { cb(nil, $0, ctx) }
            }
        }
    }
}

/// Async friends-authorization status: resolves with the raw Int32 status value
/// encoded as a JSON number string (e.g. `"3"` for `.authorized`).
@_cdecl("gk_local_player_load_friends_authorization_async")
public func gk_local_player_load_friends_authorization_async(
    _ cb: @convention(c) (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    Task {
        do {
            let status = try await withCheckedThrowingContinuation { (cont: CheckedContinuation<GKFriendsAuthorizationStatus, Error>) in
                GKLocalPlayer.local.loadFriendsAuthorizationStatus { status, error in
                    if let error { cont.resume(throwing: error) }
                    else { cont.resume(returning: status) }
                }
            }
            let json = "\(status.rawValue)"
            json.withCString { cb($0, nil, ctx) }
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

// MARK: - GKMatchmaker

/// Async findMatch: fires callback with a retained GKMatchBox pointer on success.
@_cdecl("gk_matchmaker_find_match_async")
public func gk_matchmaker_find_match_async(
    _ requestJSON: UnsafePointer<CChar>?,
    _ cb: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    Task {
        do {
            let payload = try gkDecodeJSON(requestJSON, as: GKMatchRequestPayload.self)
            let request = try await gkMakeMatchRequest(from: payload)
            let match = try await withCheckedThrowingContinuation { (cont: CheckedContinuation<GKMatch, Error>) in
                GKMatchmaker.shared().findMatch(for: request) { match, error in
                    if let error { cont.resume(throwing: error) }
                    else if let match { cont.resume(returning: match) }
                    else { cont.resume(throwing: GKBridgeError.unknown("findMatch returned nil")) }
                }
            }
            let box = gk_retain(GKMatchBox(match: match))
            cb(box, nil, ctx)
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

/// Async findPlayers (hosted): resolves with a JSON array of GKPlayerPayload.
@_cdecl("gk_matchmaker_find_players_async")
public func gk_matchmaker_find_players_async(
    _ requestJSON: UnsafePointer<CChar>?,
    _ cb: @convention(c) (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    Task {
        do {
            let payload = try gkDecodeJSON(requestJSON, as: GKMatchRequestPayload.self)
            let request = try await gkMakeMatchRequest(from: payload)
            let players = try await withCheckedThrowingContinuation { (cont: CheckedContinuation<[GKPlayer], Error>) in
                GKMatchmaker.shared().findPlayers(forHostedRequest: request) { players, error in
                    if let error { cont.resume(throwing: error) }
                    else { cont.resume(returning: players ?? []) }
                }
            }
            let json = try gkEncodeJSON(players.map(gkPlayerPayload(from:)))
            json.withCString { cb($0, nil, ctx) }
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

// MARK: - GKLeaderboard

/// Async loadLeaderboards: resolves with a JSON array of GKLeaderboardPayload.
@_cdecl("gk_leaderboard_load_async")
public func gk_leaderboard_load_async(
    _ idsJSON: UnsafePointer<CChar>?,
    _ cb: @convention(c) (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    Task {
        do {
            let ids: [String]? = idsJSON.flatMap { try? gkDecodeJSON($0, as: [String].self) }
            let leaderboards = try await gkLoadLeaderboards(with: ids)
            let json = try gkEncodeJSON(leaderboards.map(gkLeaderboardPayload(from:)))
            json.withCString { cb($0, nil, ctx) }
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

/// Async loadEntries: resolves with a GKLoadEntriesPayload JSON object.
@_cdecl("gk_leaderboard_load_entries_async")
public func gk_leaderboard_load_entries_async(
    _ leaderboardID: UnsafePointer<CChar>?,
    _ playerScope: Int32,
    _ timeScope: Int32,
    _ rangeLocation: Int,
    _ rangeLength: Int,
    _ cb: @convention(c) (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    Task {
        do {
            guard let leaderboardID else {
                throw GKBridgeError.unknown("missing leaderboard identifier")
            }
            let leaderboards = try await gkLoadLeaderboards(with: [String(cString: leaderboardID)])
            guard let leaderboard = leaderboards.first else {
                throw GKBridgeError.notFound("leaderboard not found")
            }
            let scope: GKLeaderboard.PlayerScope = playerScope == 1 ? .friendsOnly : .global
            let tscope: GKLeaderboard.TimeScope
            switch timeScope {
            case 0:  tscope = .today
            case 1:  tscope = .week
            default: tscope = .allTime
            }
            let nsRange = NSRange(location: rangeLocation, length: rangeLength)
            let result = try await withCheckedThrowingContinuation { (cont: CheckedContinuation<(GKLeaderboard.Entry?, [GKLeaderboard.Entry], Int), Error>) in
                leaderboard.loadEntries(for: scope, timeScope: tscope, range: nsRange) { local, entries, total, error in
                    if let error { cont.resume(throwing: error) }
                    else { cont.resume(returning: (local, entries ?? [], total)) }
                }
            }
            let payload = GKLoadEntriesPayload(
                localPlayerEntry: result.0.map(gkEntryPayload(from:)),
                entries: result.1.map(gkEntryPayload(from:)),
                totalPlayerCount: result.2
            )
            let json = try gkEncodeJSON(payload)
            json.withCString { cb($0, nil, ctx) }
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

// MARK: - GKAchievement

/// Async loadAchievements: resolves with a JSON array of GKAchievementPayload.
@_cdecl("gk_achievement_load_async")
public func gk_achievement_load_async(
    _ cb: @convention(c) (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    Task {
        do {
            let achievements = try await withCheckedThrowingContinuation { (cont: CheckedContinuation<[GKAchievement], Error>) in
                GKAchievement.loadAchievements { achievements, error in
                    if let error { cont.resume(throwing: error) }
                    else { cont.resume(returning: achievements ?? []) }
                }
            }
            let json = try gkEncodeJSON(achievements.map(gkAchievementPayload(from:)))
            json.withCString { cb($0, nil, ctx) }
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

/// Async report achievements: resolves with `"null"` on success.
@_cdecl("gk_achievement_report_async")
public func gk_achievement_report_async(
    _ achievementsJSON: UnsafePointer<CChar>?,
    _ cb: @convention(c) (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    Task {
        do {
            let inputs = try gkDecodeJSON(achievementsJSON, as: [GKAchievementInputPayload].self)
            let achievements = try await gkBuildAchievements(from: inputs)
            try await withCheckedThrowingContinuation { (cont: CheckedContinuation<Void, Error>) in
                GKAchievement.report(achievements) { error in
                    if let error { cont.resume(throwing: error) }
                    else { cont.resume() }
                }
            }
            "null".withCString { cb($0, nil, ctx) }
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

// MARK: - GKSavedGame

/// Async fetchSavedGames: resolves with a JSON array of GKSavedGamePayload.
@_cdecl("gk_saved_game_fetch_all_async")
public func gk_saved_game_fetch_all_async(
    _ cb: @convention(c) (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    Task {
        do {
            let savedGames = try await withCheckedThrowingContinuation { (cont: CheckedContinuation<[GKSavedGame], Error>) in
                GKLocalPlayer.local.fetchSavedGames { games, error in
                    if let error { cont.resume(throwing: error) }
                    else { cont.resume(returning: games ?? []) }
                }
            }
            let json = try gkEncodeJSON(savedGames.map(gkSavedGamePayload(from:)))
            json.withCString { cb($0, nil, ctx) }
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

/// Async loadData for a saved game: resolves with a GKBinaryPayload JSON object.
@_cdecl("gk_saved_game_load_data_async")
public func gk_saved_game_load_data_async(
    _ savedGameJSON: UnsafePointer<CChar>?,
    _ cb: @convention(c) (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    Task {
        do {
            let payload = try gkDecodeJSON(savedGameJSON, as: GKSavedGamePayload.self)
            let allGames = try await withCheckedThrowingContinuation { (cont: CheckedContinuation<[GKSavedGame], Error>) in
                GKLocalPlayer.local.fetchSavedGames { games, error in
                    if let error { cont.resume(throwing: error) }
                    else { cont.resume(returning: games ?? []) }
                }
            }
            guard let game = allGames.first(where: { savedGame in
                gkSavedGamePayload(from: savedGame).name == payload.name &&
                gkSavedGamePayload(from: savedGame).deviceName == payload.deviceName &&
                gkSavedGamePayload(from: savedGame).modificationDate == payload.modificationDate
            }) else {
                throw GKBridgeError.notFound("saved game not found")
            }
            let data = try await withCheckedThrowingContinuation { (cont: CheckedContinuation<Data, Error>) in
                game.loadData { data, error in
                    if let error { cont.resume(throwing: error) }
                    else { cont.resume(returning: data ?? Data()) }
                }
            }
            let json = try gkBinaryPayloadJSON(data)
            json.withCString { cb($0, nil, ctx) }
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

/// Async saveGameData: resolves with the new GKSavedGamePayload JSON.
@_cdecl("gk_saved_game_save_async")
public func gk_saved_game_save_async(
    _ name: UnsafePointer<CChar>?,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ cb: @convention(c) (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    Task {
        do {
            guard let name else {
                throw GKBridgeError.unknown("missing saved game name")
            }
            let payload = data.map { Data(bytes: $0, count: len) } ?? Data()
            let savedGame = try await withCheckedThrowingContinuation { (cont: CheckedContinuation<GKSavedGame, Error>) in
                GKLocalPlayer.local.saveGameData(payload, withName: String(cString: name)) { savedGame, error in
                    if let error { cont.resume(throwing: error) }
                    else if let savedGame { cont.resume(returning: savedGame) }
                    else { cont.resume(throwing: GKBridgeError.unknown("saveGameData returned nil")) }
                }
            }
            let json = try gkEncodeJSON(gkSavedGamePayload(from: savedGame))
            json.withCString { cb($0, nil, ctx) }
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}
