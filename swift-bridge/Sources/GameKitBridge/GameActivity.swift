import AppKit
import Foundation
import GameKit

#if GAMEKIT_HAS_MACOS26_SDK
struct GKGameActivityDefinitionPayload: Codable {
    let identifier: String
    let groupIdentifier: String?
    let title: String
    let details: String?
    let defaultProperties: [String: String]
    let fallbackURL: String?
    let supportsPartyCode: Bool
    let maxPlayers: Int?
    let minPlayers: Int?
    let supportsUnlimitedPlayers: Bool
    let playStyle: Int32
    let releaseState: String
}

struct GKGameActivityScorePayload: Codable {
    let leaderboardID: String
    let value: Int64
    let context: UInt64
    let playerGameID: String?
}

struct GKGameActivityAchievementInputPayload: Codable {
    let identifier: String
    let percentComplete: Double
    let showsCompletionBanner: Bool
    let playerGameID: String?
}

struct GKGameActivitySnapshotPayload: Codable {
    let identifier: String
    let activityDefinition: GKGameActivityDefinitionPayload
    let properties: [String: String]
    let state: Int32
    let partyCode: String?
    let partyURL: String?
    let creationDate: String
    let startDate: String?
    let lastResumeDate: String?
    let endDate: String?
    let durationSeconds: Double
    let achievements: [GKAchievementPayload]
    let leaderboardScores: [GKGameActivityScorePayload]
}

@available(macOS 26.0, *)
private func gkGameActivityReleaseState(_ state: GKReleaseState) -> String {
    switch state {
    case .released:
        return "released"
    case .prereleased:
        return "prereleased"
    default:
        return "unknown"
    }
}

@available(macOS 26.0, *)
private func gkGameActivityPlayStyle(_ playStyle: GKGameActivityPlayStyle) -> Int32 {
    switch playStyle {
    case .synchronous:
        return 1
    case .asynchronous:
        return 2
    default:
        return 0
    }
}

@available(macOS 26.0, *)
func gkGameActivityDefinitionPayload(from definition: GKGameActivityDefinition) -> GKGameActivityDefinitionPayload {
    GKGameActivityDefinitionPayload(
        identifier: definition.identifier,
        groupIdentifier: definition.groupIdentifier,
        title: definition.title,
        details: definition.details,
        defaultProperties: definition.defaultProperties,
        fallbackURL: definition.fallbackURL?.absoluteString,
        supportsPartyCode: definition.supportsPartyCode,
        maxPlayers: (definition.value(forKey: "maxPlayers") as? NSNumber)?.intValue,
        minPlayers: (definition.value(forKey: "minPlayers") as? NSNumber)?.intValue,
        supportsUnlimitedPlayers: definition.supportsUnlimitedPlayers,
        playStyle: gkGameActivityPlayStyle(definition.playStyle),
        releaseState: gkGameActivityReleaseState(definition.releaseState)
    )
}

@available(macOS 26.0, *)
func gkGameActivityPayload(from activity: GKGameActivity) -> GKGameActivitySnapshotPayload {
    let achievements = activity.achievements.sorted { $0.identifier < $1.identifier }
    let leaderboardScores = activity.leaderboardScores.sorted {
        let lhsKey = $0.leaderboardID + $0.player.gamePlayerID
        let rhsKey = $1.leaderboardID + $1.player.gamePlayerID
        return lhsKey < rhsKey
    }

    return GKGameActivitySnapshotPayload(
        identifier: activity.identifier,
        activityDefinition: gkGameActivityDefinitionPayload(from: activity.activityDefinition),
        properties: activity.properties,
        state: Int32(activity.state.rawValue),
        partyCode: activity.partyCode,
        partyURL: activity.partyURL?.absoluteString,
        creationDate: gkDateString(activity.creationDate) ?? "",
        startDate: gkDateString(activity.startDate),
        lastResumeDate: gkDateString(activity.lastResumeDate),
        endDate: gkDateString(activity.endDate),
        durationSeconds: activity.duration,
        achievements: achievements.map(gkAchievementPayload(from:)),
        leaderboardScores: leaderboardScores.map {
            GKGameActivityScorePayload(
                leaderboardID: $0.leaderboardID,
                value: Int64($0.value),
                context: UInt64($0.context),
                playerGameID: $0.player.gamePlayerID
            )
        }
    )
}

@available(macOS 26.0, *)
private func gkLoadGameActivityDefinitions(ids: [String]?) async throws -> [GKGameActivityDefinition] {
    try await withCheckedThrowingContinuation { continuation in
        if let ids {
            GKGameActivityDefinition.loadGameActivityDefinitions(IDs: ids) { definitions, error in
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume(returning: definitions ?? [])
                }
            }
        } else {
            GKGameActivityDefinition.loadGameActivityDefinitions { definitions, error in
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume(returning: definitions ?? [])
                }
            }
        }
    }
}

@available(macOS 26.0, *)
private func gkLoadGameActivityDefinition(identifier: String) async throws -> GKGameActivityDefinition {
    let definitions = try await gkLoadGameActivityDefinitions(ids: [identifier])
    guard let definition = definitions.first(where: { $0.identifier == identifier }) else {
        throw GKBridgeError.notFound("game activity definition not found")
    }
    return definition
}

@_cdecl("gk_game_activity_definition_load_json")
public func gk_game_activity_definition_load_json(
    _ idsJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                let ids: [String]?
                if let idsJSON {
                    ids = try gkDecodeJSON(idsJSON, as: [String].self)
                } else {
                    ids = nil
                }
                return try await gkLoadGameActivityDefinitions(ids: ids)
            },
            onSuccess: { definitions in
                outJSON?.pointee = gkCString((try? gkEncodeJSON(definitions.map(gkGameActivityDefinitionPayload(from:)))) ?? "[]")
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivityDefinition requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_definition_load_achievement_descriptions_json")
public func gk_game_activity_definition_load_achievement_descriptions_json(
    _ identifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                guard let identifier else {
                    throw GKBridgeError.unknown("missing game activity definition identifier")
                }
                let definition = try await gkLoadGameActivityDefinition(identifier: String(cString: identifier))
                return try await withCheckedThrowingContinuation {
                    (continuation: CheckedContinuation<[GKAchievementDescription], Error>) in
                    definition.loadAchievementDescriptions { descriptions, error in
                        if let error {
                            continuation.resume(throwing: error)
                        } else {
                            continuation.resume(returning: descriptions ?? [])
                        }
                    }
                }
            },
            onSuccess: { (descriptions: [GKAchievementDescription]) in
                outJSON?.pointee = gkCString((try? gkEncodeJSON(descriptions.map(gkAchievementDescriptionPayload(from:)))) ?? "[]")
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivityDefinition requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_definition_load_leaderboards_json")
public func gk_game_activity_definition_load_leaderboards_json(
    _ identifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                guard let identifier else {
                    throw GKBridgeError.unknown("missing game activity definition identifier")
                }
                let definition = try await gkLoadGameActivityDefinition(identifier: String(cString: identifier))
                return try await withCheckedThrowingContinuation {
                    (continuation: CheckedContinuation<[GKLeaderboard], Error>) in
                    definition.loadLeaderboards { leaderboards, error in
                        if let error {
                            continuation.resume(throwing: error)
                        } else {
                            continuation.resume(returning: leaderboards ?? [])
                        }
                    }
                }
            },
            onSuccess: { (leaderboards: [GKLeaderboard]) in
                outJSON?.pointee = gkCString((try? gkEncodeJSON(leaderboards.map(gkLeaderboardPayload(from:)))) ?? "[]")
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivityDefinition requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_definition_load_image_tiff_json")
public func gk_game_activity_definition_load_image_tiff_json(
    _ identifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                guard let identifier else {
                    throw GKBridgeError.unknown("missing game activity definition identifier")
                }
                let definition = try await gkLoadGameActivityDefinition(identifier: String(cString: identifier))
                let image = try await withCheckedThrowingContinuation {
                    (continuation: CheckedContinuation<NSImage, Error>) in
                    definition.loadImage { image, error in
                        if let error {
                            continuation.resume(throwing: error)
                        } else if let image {
                            continuation.resume(returning: image)
                        } else {
                            continuation.resume(throwing: GKBridgeError.unknown("game activity image load returned nil"))
                        }
                    }
                }
                guard let data = image.tiffRepresentation else {
                    throw GKBridgeError.unknown("game activity image did not expose TIFF data")
                }
                return data
            },
            onSuccess: { data in
                outJSON?.pointee = gkCString((try? gkBinaryPayloadJSON(data)) ?? "{}")
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivityDefinition requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_has_pending")
public func gk_game_activity_has_pending(
    _ outPending: UnsafeMutablePointer<Bool>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                await withCheckedContinuation { continuation in
                    GKGameActivity.checkPendingGameActivityExistence { exists in
                        continuation.resume(returning: exists)
                    }
                }
            },
            onSuccess: { exists in
                outPending?.pointee = exists
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_valid_party_code_alphabet_json")
public func gk_game_activity_valid_party_code_alphabet_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        do {
            outJSON?.pointee = gkCString(try gkEncodeJSON(GKGameActivity.validPartyCodeAlphabet))
            return GK_OK
        } catch {
            gkPopulateError(outError, with: error)
            return gkStatusFor(error)
        }
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_is_valid_party_code")
public func gk_game_activity_is_valid_party_code(
    _ partyCode: UnsafePointer<CChar>?,
    _ outValid: UnsafeMutablePointer<Bool>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        guard let partyCode else {
            gkPopulateError(outError, with: GKBridgeError.unknown("missing party code"))
            return GK_UNKNOWN
        }
        outValid?.pointee = GKGameActivity.isValidPartyCode(String(cString: partyCode))
        return GK_OK
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_create")
public func gk_game_activity_create(
    _ definitionIdentifier: UnsafePointer<CChar>?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                guard let definitionIdentifier else {
                    throw GKBridgeError.unknown("missing game activity definition identifier")
                }
                let definition = try await gkLoadGameActivityDefinition(identifier: String(cString: definitionIdentifier))
                return GKGameActivity(definition: definition)
            },
            onSuccess: { activity in
                outPtr?.pointee = gk_retain(activity)
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_start")
public func gk_game_activity_start(
    _ definitionIdentifier: UnsafePointer<CChar>?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                guard let definitionIdentifier else {
                    throw GKBridgeError.unknown("missing game activity definition identifier")
                }
                let definition = try await gkLoadGameActivityDefinition(identifier: String(cString: definitionIdentifier))
                return try GKGameActivity.start(definition: definition)
            },
            onSuccess: { activity in
                outPtr?.pointee = gk_retain(activity)
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_start_with_party_code")
public func gk_game_activity_start_with_party_code(
    _ definitionIdentifier: UnsafePointer<CChar>?,
    _ partyCode: UnsafePointer<CChar>?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                guard let definitionIdentifier else {
                    throw GKBridgeError.unknown("missing game activity definition identifier")
                }
                guard let partyCode else {
                    throw GKBridgeError.unknown("missing game activity party code")
                }
                let definition = try await gkLoadGameActivityDefinition(identifier: String(cString: definitionIdentifier))
                return try GKGameActivity.start(definition: definition, partyCode: String(cString: partyCode))
            },
            onSuccess: { activity in
                outPtr?.pointee = gk_retain(activity)
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_retain")
public func gk_game_activity_retain(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    if #available(macOS 26.0, *) {
        return gk_retain(gk_borrow(ptr, as: GKGameActivity.self))
    }
    return nil
}

@_cdecl("gk_game_activity_release")
public func gk_game_activity_release(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    if #available(macOS 26.0, *) {
        gk_release(ptr)
    }
}

@_cdecl("gk_game_activity_snapshot_json")
public func gk_game_activity_snapshot_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        do {
            let activity = gk_borrow(ptr, as: GKGameActivity.self)
            outJSON?.pointee = gkCString(try gkEncodeJSON(gkGameActivityPayload(from: activity)))
            return GK_OK
        } catch {
            gkPopulateError(outError, with: error)
            return gkStatusFor(error)
        }
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_set_properties_json")
public func gk_game_activity_set_properties_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ propertiesJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        do {
            let properties = try gkDecodeJSON(propertiesJSON, as: [String: String].self)
            let activity = gk_borrow(ptr, as: GKGameActivity.self)
            activity.properties = properties
            return GK_OK
        } catch {
            gkPopulateError(outError, with: error)
            return gkStatusFor(error)
        }
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_begin")
public func gk_game_activity_begin(
    _ ptr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        gk_borrow(ptr, as: GKGameActivity.self).start()
        return GK_OK
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_pause")
public func gk_game_activity_pause(
    _ ptr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        gk_borrow(ptr, as: GKGameActivity.self).pause()
        return GK_OK
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_resume")
public func gk_game_activity_resume(
    _ ptr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        gk_borrow(ptr, as: GKGameActivity.self).resume()
        return GK_OK
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_end")
public func gk_game_activity_end(
    _ ptr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        gk_borrow(ptr, as: GKGameActivity.self).end()
        return GK_OK
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_make_match_request_json")
public func gk_game_activity_make_match_request_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        let activity = gk_borrow(ptr, as: GKGameActivity.self)
        let payload = activity.makeMatchRequest().map(gkMatchRequestPayload(from:))
        outJSON?.pointee = gkCString((try? gkEncodeJSON(payload)) ?? "null")
        return GK_OK
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_find_match")
public func gk_game_activity_find_match(
    _ ptr: UnsafeMutableRawPointer?,
    _ outMatchPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                let activity = gk_borrow(ptr, as: GKGameActivity.self)
                return try await withCheckedThrowingContinuation { continuation in
                    activity.findMatch { match, error in
                        if let error {
                            continuation.resume(throwing: error)
                        } else if let match {
                            continuation.resume(returning: match)
                        } else {
                            continuation.resume(throwing: GKBridgeError.unknown("game activity findMatch returned nil"))
                        }
                    }
                }
            },
            onSuccess: { match in
                outMatchPtr?.pointee = gk_retain(GKMatchBox(match: match))
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_find_hosted_players_json")
public func gk_game_activity_find_hosted_players_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                let activity = gk_borrow(ptr, as: GKGameActivity.self)
                return try await withCheckedThrowingContinuation {
                    (continuation: CheckedContinuation<[GKPlayer], Error>) in
                    activity.findPlayersForHostedMatch { players, error in
                        if let error {
                            continuation.resume(throwing: error)
                        } else {
                            continuation.resume(returning: players ?? [])
                        }
                    }
                }
            },
            onSuccess: { (players: [GKPlayer]) in
                outJSON?.pointee = gkCString((try? gkEncodeJSON(players.map(gkPlayerPayload(from:)))) ?? "[]")
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_set_score_json")
public func gk_game_activity_set_score_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ scoreJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                let payload = try gkDecodeJSON(scoreJSON, as: GKGameActivityScorePayload.self)
                let activity = gk_borrow(ptr, as: GKGameActivity.self)
                guard let leaderboard = try await gkLoadLeaderboards(with: [payload.leaderboardID]).first else {
                    throw GKBridgeError.notFound("leaderboard not found")
                }
                activity.setScore(on: leaderboard, to: Int(payload.value), context: Int(payload.context))
                return ()
            },
            onSuccess: { (_: Void) in },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_remove_scores")
public func gk_game_activity_remove_scores(
    _ ptr: UnsafeMutableRawPointer?,
    _ leaderboardIDsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                let ids = try gkDecodeJSON(leaderboardIDsJSON, as: [String].self)
                let leaderboards = try await gkLoadLeaderboards(with: ids)
                let foundIDs = Set(leaderboards.map(\.baseLeaderboardID))
                if foundIDs.count != Set(ids).count {
                    throw GKBridgeError.notFound("one or more leaderboards were not found")
                }
                let activity = gk_borrow(ptr, as: GKGameActivity.self)
                activity.removeScores(from: leaderboards)
                return ()
            },
            onSuccess: { (_: Void) in },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_set_progress_json")
public func gk_game_activity_set_progress_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ achievementJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        do {
            let payload = try gkDecodeJSON(achievementJSON, as: GKGameActivityAchievementInputPayload.self)
            let activity = gk_borrow(ptr, as: GKGameActivity.self)
            let achievement = GKAchievement(identifier: payload.identifier)
            achievement.showsCompletionBanner = payload.showsCompletionBanner
            activity.setProgress(on: achievement, to: payload.percentComplete)
            return GK_OK
        } catch {
            gkPopulateError(outError, with: error)
            return gkStatusFor(error)
        }
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_complete_achievement_json")
public func gk_game_activity_complete_achievement_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ achievementJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        do {
            let payload = try gkDecodeJSON(achievementJSON, as: GKGameActivityAchievementInputPayload.self)
            let activity = gk_borrow(ptr, as: GKGameActivity.self)
            let achievement = GKAchievement(identifier: payload.identifier)
            achievement.showsCompletionBanner = payload.showsCompletionBanner
            activity.setAchievementCompleted(achievement)
            return GK_OK
        } catch {
            gkPopulateError(outError, with: error)
            return gkStatusFor(error)
        }
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_remove_achievements_json")
public func gk_game_activity_remove_achievements_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ achievementsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null game activity pointer"))
        return GK_UNKNOWN
    }

    if #available(macOS 26.0, *) {
        do {
            let payloads = try gkDecodeJSON(achievementsJSON, as: [GKGameActivityAchievementInputPayload].self)
            let activity = gk_borrow(ptr, as: GKGameActivity.self)
            let achievements = payloads.map { GKAchievement(identifier: $0.identifier) }
            activity.removeAchievements(achievements)
            return GK_OK
        } catch {
            gkPopulateError(outError, with: error)
            return gkStatusFor(error)
        }
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKGameActivity requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}
#else
struct GKGameActivityDefinitionPayload: Codable {}
struct GKGameActivityScorePayload: Codable {}
struct GKGameActivityAchievementInputPayload: Codable {}
struct GKGameActivitySnapshotPayload: Codable {}

private func gkGameActivityUnavailable(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ message: String = "GKGameActivity requires the macOS 26 SDK"
) -> Int32 {
    gkPopulateError(outError, with: GKBridgeError.unavailable(message))
    return GK_UNAVAILABLE
}

@_cdecl("gk_game_activity_definition_load_json")
public func gk_game_activity_definition_load_json(
    _ idsJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_definition_load_achievement_descriptions_json")
public func gk_game_activity_definition_load_achievement_descriptions_json(
    _ identifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_definition_load_leaderboards_json")
public func gk_game_activity_definition_load_leaderboards_json(
    _ identifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_definition_load_image_tiff_json")
public func gk_game_activity_definition_load_image_tiff_json(
    _ identifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_has_pending")
public func gk_game_activity_has_pending(
    _ outPending: UnsafeMutablePointer<Bool>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_valid_party_code_alphabet_json")
public func gk_game_activity_valid_party_code_alphabet_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_is_valid_party_code")
public func gk_game_activity_is_valid_party_code(
    _ partyCode: UnsafePointer<CChar>?,
    _ outValid: UnsafeMutablePointer<Bool>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_create")
public func gk_game_activity_create(
    _ definitionIdentifier: UnsafePointer<CChar>?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_start")
public func gk_game_activity_start(
    _ definitionIdentifier: UnsafePointer<CChar>?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_start_with_party_code")
public func gk_game_activity_start_with_party_code(
    _ definitionIdentifier: UnsafePointer<CChar>?,
    _ partyCode: UnsafePointer<CChar>?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_retain")
public func gk_game_activity_retain(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? { nil }

@_cdecl("gk_game_activity_release")
public func gk_game_activity_release(_ ptr: UnsafeMutableRawPointer?) {}

@_cdecl("gk_game_activity_snapshot_json")
public func gk_game_activity_snapshot_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_set_properties_json")
public func gk_game_activity_set_properties_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ propertiesJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_begin")
public func gk_game_activity_begin(
    _ ptr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_pause")
public func gk_game_activity_pause(
    _ ptr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_resume")
public func gk_game_activity_resume(
    _ ptr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_end")
public func gk_game_activity_end(
    _ ptr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_make_match_request_json")
public func gk_game_activity_make_match_request_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_find_match")
public func gk_game_activity_find_match(
    _ ptr: UnsafeMutableRawPointer?,
    _ outMatchPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_find_hosted_players_json")
public func gk_game_activity_find_hosted_players_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_set_score_json")
public func gk_game_activity_set_score_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ scoreJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_remove_scores")
public func gk_game_activity_remove_scores(
    _ ptr: UnsafeMutableRawPointer?,
    _ leaderboardIDsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_set_progress_json")
public func gk_game_activity_set_progress_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ achievementJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_complete_achievement_json")
public func gk_game_activity_complete_achievement_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ achievementJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }

@_cdecl("gk_game_activity_remove_achievements_json")
public func gk_game_activity_remove_achievements_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ achievementsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 { gkGameActivityUnavailable(outError) }
#endif
