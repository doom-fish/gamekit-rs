import Foundation
import GameKit

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

func gkEntryPayload(from entry: GKLeaderboard.Entry) -> GKLeaderboardEntryPayload {
    GKLeaderboardEntryPayload(
        rank: entry.rank,
        score: entry.score,
        formattedScore: entry.formattedScore,
        context: UInt(entry.context),
        date: gkDateString(entry.date) ?? "",
        player: gkPlayerPayload(from: entry.player)
    )
}

private func gkTimeScope(from value: Int32) -> GKLeaderboard.TimeScope {
    switch value {
    case 0:
        return .today
    case 1:
        return .week
    default:
        return .allTime
    }
}

private func gkLoadLeaderboard(with identifier: String) async throws -> GKLeaderboard {
    let leaderboards = try await gkLoadLeaderboards(with: [identifier])
    guard let leaderboard = leaderboards.first else {
        throw GKBridgeError.notFound("leaderboard not found")
    }
    return leaderboard
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
    gkBlockOnAsync(
        work: {
            guard let leaderboardID else {
                throw GKBridgeError.unknown("missing leaderboard identifier")
            }
            let leaderboard = try await gkLoadLeaderboard(with: String(cString: leaderboardID))
            let scope: GKLeaderboard.PlayerScope = playerScope == 1 ? .friendsOnly : .global
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<(GKLeaderboard.Entry?, [GKLeaderboard.Entry], Int), Error>) in
                leaderboard.loadEntries(
                    for: scope,
                    timeScope: gkTimeScope(from: timeScope),
                    range: NSRange(location: rangeLocation, length: rangeLength)
                ) { localEntry, entries, totalPlayers, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: (localEntry, entries ?? [], totalPlayers))
                    }
                }
            }
        },
        onSuccess: { (result: (GKLeaderboard.Entry?, [GKLeaderboard.Entry], Int)) in
            let payload = GKLoadEntriesPayload(
                localPlayerEntry: result.0.map(gkEntryPayload(from:)),
                entries: result.1.map(gkEntryPayload(from:)),
                totalPlayerCount: result.2
            )
            outJSON?.pointee = gkCString((try? gkEncodeJSON(payload)) ?? "{}")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_leaderboard_load_entries_for_players_json")
public func gk_leaderboard_load_entries_for_players_json(
    _ leaderboardID: UnsafePointer<CChar>?,
    _ playerIDsJSON: UnsafePointer<CChar>?,
    _ timeScope: Int32,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let leaderboardID else {
                throw GKBridgeError.unknown("missing leaderboard identifier")
            }
            let leaderboard = try await gkLoadLeaderboard(with: String(cString: leaderboardID))
            let playerIDs = try gkDecodeJSON(playerIDsJSON, as: [String].self)
            let players = try await gkLoadPlayers(identifiedBy: playerIDs)
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<(GKLeaderboard.Entry?, [GKLeaderboard.Entry]), Error>) in
                leaderboard.loadEntries(for: players, timeScope: gkTimeScope(from: timeScope)) { localEntry, entries, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: (localEntry, entries ?? []))
                    }
                }
            }
        },
        onSuccess: { (result: (GKLeaderboard.Entry?, [GKLeaderboard.Entry])) in
            let payload = GKLoadEntriesPayload(
                localPlayerEntry: result.0.map(gkEntryPayload(from:)),
                entries: result.1.map(gkEntryPayload(from:)),
                totalPlayerCount: result.1.count
            )
            outJSON?.pointee = gkCString((try? gkEncodeJSON(payload)) ?? "{}")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}
