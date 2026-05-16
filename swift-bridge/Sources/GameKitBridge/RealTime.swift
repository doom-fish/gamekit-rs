import Foundation
import GameKit

struct GKMatchRequestPayload: Codable {
    let minPlayers: Int
    let maxPlayers: Int
    let playerGroup: Int
    let playerAttributes: UInt32
    let recipientIds: [String]
    let inviteMessage: String?
    let defaultNumberOfPlayers: Int
}

func gkMakeMatchRequest(from payload: GKMatchRequestPayload) async throws -> GKMatchRequest {
    let request = GKMatchRequest()
    request.minPlayers = payload.minPlayers
    request.maxPlayers = payload.maxPlayers
    request.playerGroup = payload.playerGroup
    request.playerAttributes = payload.playerAttributes
    request.defaultNumberOfPlayers = payload.defaultNumberOfPlayers
    request.inviteMessage = payload.inviteMessage
    if !payload.recipientIds.isEmpty {
        request.recipients = try await gkLoadPlayers(identifiedBy: payload.recipientIds)
    }
    return request
}

@_cdecl("gk_matchmaker_find_match_json")
public func gk_matchmaker_find_match_json(
    _ requestJSON: UnsafePointer<CChar>?,
    _ outMatchPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let payload = try gkDecodeJSON(requestJSON, as: GKMatchRequestPayload.self)
            let request = try await gkMakeMatchRequest(from: payload)
            return try await withCheckedThrowingContinuation { continuation in
                GKMatchmaker.shared().findMatch(for: request) { match, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else if let match {
                        continuation.resume(returning: match)
                    } else {
                        continuation.resume(throwing: GKBridgeError.unknown("findMatch returned nil"))
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

@_cdecl("gk_matchmaker_find_hosted_players_json")
public func gk_matchmaker_find_hosted_players_json(
    _ requestJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let payload = try gkDecodeJSON(requestJSON, as: GKMatchRequestPayload.self)
            let request = try await gkMakeMatchRequest(from: payload)
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKPlayer], Error>) in
                GKMatchmaker.shared().findPlayers(forHostedRequest: request) { players, error in
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

@_cdecl("gk_matchmaker_add_players_to_match")
public func gk_matchmaker_add_players_to_match(
    _ ptr: UnsafeMutableRawPointer?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null match pointer"))
        return GK_UNKNOWN
    }

    return gkBlockOnAsync(
        work: {
            let payload = try gkDecodeJSON(requestJSON, as: GKMatchRequestPayload.self)
            let request = try await gkMakeMatchRequest(from: payload)
            let box = gk_borrow(ptr, as: GKMatchBox.self)
            return try await withCheckedThrowingContinuation { continuation in
                GKMatchmaker.shared().addPlayers(to: box.match, matchRequest: request) { error in
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

@_cdecl("gk_matchmaker_cancel")
public func gk_matchmaker_cancel() {
    GKMatchmaker.shared().cancel()
}

@_cdecl("gk_matchmaker_finish")
public func gk_matchmaker_finish(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    let box = gk_borrow(ptr, as: GKMatchBox.self)
    GKMatchmaker.shared().finishMatchmaking(for: box.match)
}

@_cdecl("gk_matchmaker_query_player_group_activity")
public func gk_matchmaker_query_player_group_activity(
    _ playerGroup: Int,
    _ outActivity: UnsafeMutablePointer<Int64>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Int, Error>) in
                GKMatchmaker.shared().queryPlayerGroupActivity(playerGroup) { activity, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: activity)
                    }
                }
            }
        },
        onSuccess: { (activity: Int) in
            outActivity?.pointee = Int64(activity)
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_matchmaker_query_activity")
public func gk_matchmaker_query_activity(
    _ outActivity: UnsafeMutablePointer<Int64>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Int, Error>) in
                GKMatchmaker.shared().queryActivity { activity, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: activity)
                    }
                }
            }
        },
        onSuccess: { (activity: Int) in
            outActivity?.pointee = Int64(activity)
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_matchmaker_max_players_allowed")
public func gk_matchmaker_max_players_allowed(_ matchType: Int32) -> Int {
    let type: GKMatchType
    switch matchType {
    case 1:
        type = .hosted
    case 2:
        type = .turnBased
    default:
        type = .peerToPeer
    }
    switch type {
    case .peerToPeer:
        return 16
    case .hosted:
        return 16
    case .turnBased:
        return 16
    @unknown default:
        return 16
    }
}
