import Foundation
import GameKit

struct GKLeaderboardPayload: Codable {
    let baseLeaderboardID: String
    let title: String?
    let leaderboardType: String
}

struct GKLeaderboardEntryPayload: Codable {
    let rank: Int
    let score: Int
    let formattedScore: String
    let context: UInt
    let date: String
    let player: GKPlayerPayload
}

struct GKLoadEntriesPayload: Codable {
    let localPlayerEntry: GKLeaderboardEntryPayload?
    let entries: [GKLeaderboardEntryPayload]
    let totalPlayerCount: Int
}

private let gkDateFormatter: ISO8601DateFormatter = {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return formatter
}()

func gkLeaderboardPayload(from leaderboard: GKLeaderboard) -> GKLeaderboardPayload {
    let leaderboardType: String
    switch leaderboard.type {
    case .classic:
        leaderboardType = "classic"
    case .recurring:
        leaderboardType = "recurring"
    @unknown default:
        leaderboardType = "classic"
    }
    
    return GKLeaderboardPayload(
        baseLeaderboardID: leaderboard.baseLeaderboardID,
        title: leaderboard.title,
        leaderboardType: leaderboardType
    )
}

func gkEntryPayload(from entry: GKLeaderboard.Entry) -> GKLeaderboardEntryPayload {
    GKLeaderboardEntryPayload(
        rank: entry.rank,
        score: entry.score,
        formattedScore: entry.formattedScore,
        context: UInt(entry.context),
        date: gkDateFormatter.string(from: entry.date),
        player: gkPlayerPayload(from: entry.player)
    )
}

@_cdecl("gk_leaderboard_load_json")
public func gk_leaderboard_load_json(
    _ idsJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    return gkBlockOnAsync(
        work: {
            let ids = try gkDecodeJSON(idsJSON, as: [String].self)
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKLeaderboard], Error>) in
                GKLeaderboard.loadLeaderboards(IDs: ids) { leaderboards, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                    } else if let leaderboards = leaderboards {
                        continuation.resume(returning: leaderboards)
                    } else {
                        continuation.resume(throwing: GKBridgeError.unknown("loadLeaderboards returned nil"))
                    }
                }
            }
        },
        onSuccess: { (leaderboards: [GKLeaderboard]) in
            let payloads = leaderboards.map(gkLeaderboardPayload(from:))
            if let json = try? gkEncodeJSON(payloads) {
                outJSON?.pointee = gkCString(json)
            }
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_leaderboard_submit_score")
public func gk_leaderboard_submit_score(
    _ score: Int64,
    _ context: UInt64,
    _ idsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    return gkBlockOnAsync(
        work: {
            let ids = try gkDecodeJSON(idsJSON, as: [String].self)
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                GKLeaderboard.submitScore(
                    Int(score),
                    context: Int(context),
                    player: GKLocalPlayer.local,
                    leaderboardIDs: ids
                ) { error in
                    if let error = error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: ())
                    }
                }
            }
        },
        onSuccess: { (_: Void) in },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_leaderboard_load_entries_json")
public func gk_leaderboard_load_entries_json(
    _ leaderboardID: UnsafePointer<CChar>?,
    _ playerScope: Int32,
    _ timeScope: Int32,
    _ rangeLocation: Int,
    _ rangeLength: Int,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    return gkBlockOnAsync(
        work: {
            guard let leaderboardID = leaderboardID else {
                throw GKBridgeError.unknown("missing leaderboard ID")
            }
            let leaderboardIDStr = String(cString: leaderboardID)
            
            let scope: GKLeaderboard.PlayerScope = playerScope == 1 ? .friendsOnly : .global
            
            let time: GKLeaderboard.TimeScope
            switch timeScope {
            case 0:
                time = .today
            case 1:
                time = .week
            default:
                time = .allTime
            }
            
            let leaderboard: GKLeaderboard = try await withCheckedThrowingContinuation { continuation in
                GKLeaderboard.loadLeaderboards(IDs: [leaderboardIDStr]) { leaderboards, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                    } else if let leaderboard = leaderboards?.first {
                        continuation.resume(returning: leaderboard)
                    } else {
                        continuation.resume(throwing: GKBridgeError.notFound("leaderboard not found"))
                    }
                }
            }
            
            let result: (localPlayerEntry: GKLeaderboard.Entry?, entries: [GKLeaderboard.Entry], totalPlayerCount: Int) = try await withCheckedThrowingContinuation { continuation in
                leaderboard.loadEntries(
                    for: scope,
                    timeScope: time,
                    range: NSRange(location: rangeLocation, length: rangeLength)
                ) { localEntry, entries, totalPlayers, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: (localEntry, entries ?? [], totalPlayers))
                    }
                }
            }
            
            return result
        },
        onSuccess: { result in
            let payload = GKLoadEntriesPayload(
                localPlayerEntry: result.localPlayerEntry.map(gkEntryPayload(from:)),
                entries: result.entries.map(gkEntryPayload(from:)),
                totalPlayerCount: result.totalPlayerCount
            )
            if let json = try? gkEncodeJSON(payload) {
                outJSON?.pointee = gkCString(json)
            }
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}
