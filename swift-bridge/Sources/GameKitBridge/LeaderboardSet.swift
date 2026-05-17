import AppKit
import Foundation
import GameKit

struct GKLeaderboardSetPayload: Codable {
    let title: String
    let groupIdentifier: String?
    let identifier: String?
}

func gkLeaderboardSetPayload(from set: GKLeaderboardSet) -> GKLeaderboardSetPayload {
    GKLeaderboardSetPayload(
        title: set.title,
        groupIdentifier: set.groupIdentifier,
        identifier: set.identifier
    )
}

private func gkLoadLeaderboardSet(with identifier: String) async throws -> GKLeaderboardSet {
    try await withCheckedThrowingContinuation { continuation in
        GKLeaderboardSet.loadLeaderboardSets { sets, error in
            if let error {
                continuation.resume(throwing: error)
            } else if let set = sets?.first(where: { $0.identifier == identifier }) {
                continuation.resume(returning: set)
            } else {
                continuation.resume(throwing: GKBridgeError.notFound("leaderboard set '\(identifier)' not found"))
            }
        }
    }
}

@_cdecl("gk_leaderboard_sets_json")
public func gk_leaderboard_sets_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKLeaderboardSet], Error>) in
                GKLeaderboardSet.loadLeaderboardSets { sets, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: sets ?? [])
                    }
                }
            }
        },
        onSuccess: { (sets: [GKLeaderboardSet]) in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(sets.map(gkLeaderboardSetPayload(from:)))) ?? "[]")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_leaderboard_set_load_leaderboards_json")
public func gk_leaderboard_set_load_leaderboards_json(
    _ identifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let identifier else {
                throw GKBridgeError.unknown("missing leaderboard set identifier")
            }
            let set = try await gkLoadLeaderboardSet(with: String(cString: identifier))
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKLeaderboard], Error>) in
                set.loadLeaderboards { leaderboards, error in
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

@_cdecl("gk_leaderboard_set_load_image_json")
public func gk_leaderboard_set_load_image_json(
    _ identifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let identifier else {
                throw GKBridgeError.unknown("missing leaderboard set identifier")
            }
            let set = try await gkLoadLeaderboardSet(with: String(cString: identifier))
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Data, Error>) in
                set.loadImage { image, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else if let data = image?.tiffRepresentation {
                        continuation.resume(returning: data)
                    } else {
                        continuation.resume(throwing: GKBridgeError.notFound("leaderboard set image not available"))
                    }
                }
            }
        },
        onSuccess: { data in
            outJSON?.pointee = gkCString((try? gkBinaryPayloadJSON(data)) ?? "{}")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}
