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

struct GKMatchedPlayerPropertiesPayload: Codable {
    let player: GKPlayerPayload
    let propertiesJson: String?
}

struct GKMatchedPlayersPayload: Codable {
    let propertiesJson: String?
    let players: [GKPlayerPayload]
    let playerProperties: [GKMatchedPlayerPropertiesPayload]
}

public typealias GKInviteRecipientResponseCallbackFn = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<CChar>?,
    Int32
) -> Void

private func gkEncodeJSONObject(_ object: Any) throws -> String {
    let data = try JSONSerialization.data(withJSONObject: object, options: [.sortedKeys])
    guard let string = String(data: data, encoding: .utf8) else {
        throw GKBridgeError.unknown("failed to encode JSON object as UTF-8")
    }
    return string
}

private func gkMatchPropertiesJSONString(_ properties: [String: Any]?) -> String? {
    guard let properties else {
        return nil
    }
    return try? gkEncodeJSONObject(properties)
}

@available(macOS 14.2, *)
private func gkMatchedPlayersPayload(from matchedPlayers: GKMatchedPlayers) -> GKMatchedPlayersPayload {
    GKMatchedPlayersPayload(
        propertiesJson: gkMatchPropertiesJSONString(matchedPlayers.properties),
        players: matchedPlayers.players.map(gkPlayerPayload(from:)),
        playerProperties: matchedPlayers.playerProperties?.map {
            GKMatchedPlayerPropertiesPayload(
                player: gkPlayerPayload(from: $0.key),
                propertiesJson: gkMatchPropertiesJSONString($0.value)
            )
        } ?? []
    )
}

func gkMakeMatchRequest(
    from payload: GKMatchRequestPayload,
    responseCallback: GKInviteRecipientResponseCallbackFn? = nil,
    refcon: UnsafeMutableRawPointer? = nil
) async throws -> GKMatchRequest {
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
    if let responseCallback {
        request.recipientResponseHandler = { player, response in
            guard let json = try? gkEncodeJSON(gkPlayerPayload(from: player)), let cString = gkCString(json) else {
                responseCallback(refcon, nil, Int32(response.rawValue))
                return
            }
            defer { free(cString) }
            responseCallback(refcon, UnsafePointer(cString), Int32(response.rawValue))
        }
    }
    return request
}

@_cdecl("gk_matchmaker_find_match_json")
public func gk_matchmaker_find_match_json(
    _ requestJSON: UnsafePointer<CChar>?,
    _ callback: GKInviteRecipientResponseCallbackFn?,
    _ refcon: UnsafeMutableRawPointer?,
    _ outMatchPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let payload = try gkDecodeJSON(requestJSON, as: GKMatchRequestPayload.self)
            let request = try await gkMakeMatchRequest(
                from: payload,
                responseCallback: callback,
                refcon: refcon
            )
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
    _ callback: GKInviteRecipientResponseCallbackFn?,
    _ refcon: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let payload = try gkDecodeJSON(requestJSON, as: GKMatchRequestPayload.self)
            let request = try await gkMakeMatchRequest(
                from: payload,
                responseCallback: callback,
                refcon: refcon
            )
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

@_cdecl("gk_matchmaker_find_matched_players_json")
public func gk_matchmaker_find_matched_players_json(
    _ requestJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard #available(macOS 14.2, *) else {
                throw GKBridgeError.unavailable("rule-based matched players require macOS 14.2 or newer")
            }
            let payload = try gkDecodeJSON(requestJSON, as: GKMatchRequestPayload.self)
            let request = try await gkMakeMatchRequest(from: payload)
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<GKMatchedPlayers, Error>) in
                GKMatchmaker.shared().findMatchedPlayers(request) { matchedPlayers, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else if let matchedPlayers {
                        continuation.resume(returning: matchedPlayers)
                    } else {
                        continuation.resume(throwing: GKBridgeError.unknown("findMatchedPlayers returned nil"))
                    }
                }
            }
        },
        onSuccess: { matchedPlayers in
            if #available(macOS 14.2, *) {
                outJSON?.pointee = gkCString((try? gkEncodeJSON(gkMatchedPlayersPayload(from: matchedPlayers))) ?? "{}")
            } else {
                outJSON?.pointee = gkCString("{}")
            }
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
