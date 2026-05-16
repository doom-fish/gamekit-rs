import Foundation
import GameKit

struct GKLeaderboardPayload: Codable {
    let baseLeaderboardID: String
    let title: String?
    let groupIdentifier: String?
    let leaderboardType: String
    let startDate: String?
    let nextStartDate: String?
    let durationSeconds: Double?
}

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
        groupIdentifier: leaderboard.groupIdentifier,
        leaderboardType: leaderboardType,
        startDate: gkDateString(leaderboard.startDate),
        nextStartDate: gkDateString(leaderboard.nextStartDate),
        durationSeconds: leaderboard.type == .recurring ? leaderboard.duration : nil
    )
}

func gkLoadLeaderboards(with ids: [String]?) async throws -> [GKLeaderboard] {
    try await withCheckedThrowingContinuation { continuation in
        GKLeaderboard.loadLeaderboards(IDs: ids) { leaderboards, error in
            if let error {
                continuation.resume(throwing: error)
            } else {
                continuation.resume(returning: leaderboards ?? [])
            }
        }
    }
}

@_cdecl("gk_leaderboard_load_json")
public func gk_leaderboard_load_json(
    _ idsJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let ids: [String]?
            if let idsJSON {
                ids = try gkDecodeJSON(idsJSON, as: [String].self)
            } else {
                ids = nil
            }
            return try await gkLoadLeaderboards(with: ids)
        },
        onSuccess: { (leaderboards: [GKLeaderboard]) in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(leaderboards.map(gkLeaderboardPayload(from:)))) ?? "[]")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_leaderboard_load_previous_occurrence_json")
public func gk_leaderboard_load_previous_occurrence_json(
    _ leaderboardID: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let leaderboardID else {
                throw GKBridgeError.unknown("missing leaderboard identifier")
            }
            let identifier = String(cString: leaderboardID)
            guard let leaderboard = try await gkLoadLeaderboards(with: [identifier]).first else {
                throw GKBridgeError.notFound("leaderboard not found")
            }
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<GKLeaderboard?, Error>) in
                leaderboard.loadPreviousOccurrence { occurrence, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: occurrence)
                    }
                }
            }
        },
        onSuccess: { (leaderboard: GKLeaderboard?) in
            let payload = leaderboard.map(gkLeaderboardPayload(from:))
            outJSON?.pointee = gkCString((try? gkEncodeJSON(payload)) ?? "null")
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
    gkBlockOnAsync(
        work: {
            let ids = try gkDecodeJSON(idsJSON, as: [String].self)
            return try await withCheckedThrowingContinuation { continuation in
                GKLeaderboard.submitScore(Int(score), context: Int(context), player: GKLocalPlayer.local, leaderboardIDs: ids) { error in
                    if let error {
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

@_cdecl("gk_leaderboard_submit_score_for_id")
public func gk_leaderboard_submit_score_for_id(
    _ score: Int64,
    _ context: UInt64,
    _ leaderboardID: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let leaderboardID else {
                throw GKBridgeError.unknown("missing leaderboard identifier")
            }
            let identifier = String(cString: leaderboardID)
            guard let leaderboard = try await gkLoadLeaderboards(with: [identifier]).first else {
                throw GKBridgeError.notFound("leaderboard not found")
            }
            return try await withCheckedThrowingContinuation { continuation in
                leaderboard.submitScore(Int(score), context: Int(context), player: GKLocalPlayer.local) { error in
                    if let error {
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
