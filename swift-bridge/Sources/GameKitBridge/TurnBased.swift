import Foundation
import GameKit

struct GKTurnBasedMatchRequestPayload: Codable {
    let minPlayers: Int
    let maxPlayers: Int
    let playerGroup: Int
    let playerAttributes: UInt32
    let recipientIds: [String]
}

struct GKTurnBasedParticipantPayload: Codable {
    let index: Int
    let player: GKPlayerPayload?
    let lastTurnDate: String?
    let status: Int32
    let matchOutcome: Int32
    let timeoutDate: String?
}

struct GKTurnBasedExchangeReplyPayload: Codable {
    let recipientIndex: Int?
    let message: String?
    let dataLen: Int
    let replyDate: String?
}

struct GKTurnBasedExchangePayload: Codable {
    let index: Int
    let exchangeID: String?
    let senderIndex: Int?
    let recipientIndices: [Int]
    let status: Int32
    let message: String?
    let dataLen: Int
    let sendDate: String?
    let timeoutDate: String?
    let completionDate: String?
    let replies: [GKTurnBasedExchangeReplyPayload]
}

struct GKTurnBasedMatchPayload: Codable {
    let matchID: String
    let creationDate: String?
    let participants: [GKTurnBasedParticipantPayload]
    let status: Int32
    let currentParticipantIndex: Int?
    let message: String?
    let matchDataLen: Int
    let matchDataMaximumSize: Int
    let exchanges: [GKTurnBasedExchangePayload]
    let activeExchangeIndices: [Int]
    let completedExchangeIndices: [Int]
    let exchangeDataMaximumSize: Int
    let exchangeMaxInitiatedExchangesPerPlayer: Int
}

private func gkMakeTurnBasedRequest(from payload: GKTurnBasedMatchRequestPayload) async throws -> GKMatchRequest {
    let request = GKMatchRequest()
    request.minPlayers = payload.minPlayers
    request.maxPlayers = payload.maxPlayers
    request.playerGroup = payload.playerGroup
    request.playerAttributes = payload.playerAttributes
    if !payload.recipientIds.isEmpty {
        request.recipients = try await gkLoadPlayers(identifiedBy: payload.recipientIds)
    }
    return request
}

private func gkLoadTurnBasedMatch(with identifier: String) async throws -> GKTurnBasedMatch {
    try await withCheckedThrowingContinuation { continuation in
        GKTurnBasedMatch.load(withID: identifier) { match, error in
            if let error {
                continuation.resume(throwing: error)
            } else if let match {
                continuation.resume(returning: match)
            } else {
                continuation.resume(throwing: GKBridgeError.notFound("turn-based match not found"))
            }
        }
    }
}

private func gkParticipantIndex(_ participant: GKTurnBasedParticipant?, in participants: [GKTurnBasedParticipant]) -> Int? {
    guard let participant else {
        return nil
    }
    return participants.firstIndex { $0 === participant }
}

private func gkTurnBasedParticipantPayload(
    from participant: GKTurnBasedParticipant,
    index: Int
) -> GKTurnBasedParticipantPayload {
    GKTurnBasedParticipantPayload(
        index: index,
        player: participant.player.map(gkPlayerPayload(from:)),
        lastTurnDate: gkDateString(participant.lastTurnDate),
        status: Int32(participant.status.rawValue),
        matchOutcome: Int32(participant.matchOutcome.rawValue),
        timeoutDate: gkDateString(participant.timeoutDate)
    )
}

private func gkTurnBasedExchangeReplyPayload(
    from reply: GKTurnBasedExchangeReply,
    participants: [GKTurnBasedParticipant]
) -> GKTurnBasedExchangeReplyPayload {
    GKTurnBasedExchangeReplyPayload(
        recipientIndex: gkParticipantIndex(reply.recipient, in: participants),
        message: reply.message,
        dataLen: reply.data?.count ?? 0,
        replyDate: gkDateString(reply.replyDate)
    )
}

private func gkTurnBasedExchangePayload(
    from exchange: GKTurnBasedExchange,
    index: Int,
    participants: [GKTurnBasedParticipant]
) -> GKTurnBasedExchangePayload {
    GKTurnBasedExchangePayload(
        index: index,
        exchangeID: exchange.exchangeID,
        senderIndex: gkParticipantIndex(exchange.sender, in: participants),
        recipientIndices: exchange.recipients.compactMap { gkParticipantIndex($0, in: participants) },
        status: Int32(exchange.status.rawValue),
        message: exchange.message,
        dataLen: exchange.data?.count ?? 0,
        sendDate: gkDateString(exchange.sendDate),
        timeoutDate: gkDateString(exchange.timeoutDate),
        completionDate: gkDateString(exchange.completionDate),
        replies: (exchange.replies ?? []).map { gkTurnBasedExchangeReplyPayload(from: $0, participants: participants) }
    )
}

private func gkTurnBasedMatchPayload(from match: GKTurnBasedMatch) throws -> GKTurnBasedMatchPayload {
    let matchID = match.matchID
    let participants = match.participants
    let exchanges = match.exchanges ?? []
    let exchangePayloads = exchanges.enumerated().map { gkTurnBasedExchangePayload(from: $0.element, index: $0.offset, participants: participants) }
    let activeIDs = Set((match.activeExchanges ?? []).compactMap(\.exchangeID))
    let completedIDs = Set((match.completedExchanges ?? []).compactMap(\.exchangeID))
    return GKTurnBasedMatchPayload(
        matchID: matchID,
        creationDate: gkDateString(match.creationDate),
        participants: participants.enumerated().map { gkTurnBasedParticipantPayload(from: $0.element, index: $0.offset) },
        status: Int32(match.status.rawValue),
        currentParticipantIndex: gkParticipantIndex(match.currentParticipant, in: participants),
        message: match.message,
        matchDataLen: match.matchData?.count ?? 0,
        matchDataMaximumSize: match.matchDataMaximumSize,
        exchanges: exchangePayloads,
        activeExchangeIndices: exchangePayloads.compactMap { payload in
            guard let exchangeID = payload.exchangeID, activeIDs.contains(exchangeID) else { return nil }
            return payload.index
        },
        completedExchangeIndices: exchangePayloads.compactMap { payload in
            guard let exchangeID = payload.exchangeID, completedIDs.contains(exchangeID) else { return nil }
            return payload.index
        },
        exchangeDataMaximumSize: match.exchangeDataMaximumSize,
        exchangeMaxInitiatedExchangesPerPlayer: match.exchangeMaxInitiatedExchangesPerPlayer
    )
}

private func gkTurnBasedParticipants(
    from indices: [Int],
    in match: GKTurnBasedMatch
) throws -> [GKTurnBasedParticipant] {
    let participants = match.participants
    return try indices.map { index in
        guard participants.indices.contains(index) else {
            throw GKBridgeError.notFound("turn-based participant index \(index) is out of range")
        }
        return participants[index]
    }
}

private func gkTurnBasedExchange(at index: Int, in match: GKTurnBasedMatch) throws -> GKTurnBasedExchange {
    let exchanges = match.exchanges ?? []
    guard exchanges.indices.contains(index) else {
        throw GKBridgeError.notFound("turn-based exchange index \(index) is out of range")
    }
    return exchanges[index]
}

private func gkBuildAchievements(
    from inputs: [GKAchievementInputPayload],
    match: GKTurnBasedMatch
) async throws -> [GKAchievement] {
    let preferredPlayers = match.participants.compactMap(\.player)
    return try await gkBuildAchievements(from: inputs, preferredPlayers: preferredPlayers)
}

@_cdecl("gk_turn_based_find_match_json")
public func gk_turn_based_find_match_json(
    _ requestJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let payload = try gkDecodeJSON(requestJSON, as: GKTurnBasedMatchRequestPayload.self)
            let request = try await gkMakeTurnBasedRequest(from: payload)
            return try await withCheckedThrowingContinuation { continuation in
                GKTurnBasedMatch.find(for: request) { match, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else if let match {
                        continuation.resume(returning: match)
                    } else {
                        continuation.resume(throwing: GKBridgeError.unknown("turn-based findMatch returned nil"))
                    }
                }
            }
        },
        onSuccess: { match in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(try gkTurnBasedMatchPayload(from: match))) ?? "{}")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_turn_based_load_matches_json")
public func gk_turn_based_load_matches_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKTurnBasedMatch], Error>) in
                GKTurnBasedMatch.loadMatches { matches, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: matches ?? [])
                    }
                }
            }
        },
        onSuccess: { (matches: [GKTurnBasedMatch]) in
            let payloads = matches.compactMap { try? gkTurnBasedMatchPayload(from: $0) }
            outJSON?.pointee = gkCString((try? gkEncodeJSON(payloads)) ?? "[]")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_turn_based_load_match_json")
public func gk_turn_based_load_match_json(
    _ matchID: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            return try await gkLoadTurnBasedMatch(with: String(cString: matchID))
        },
        onSuccess: { match in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(try gkTurnBasedMatchPayload(from: match))) ?? "{}")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_turn_based_rematch_json")
public func gk_turn_based_rematch_json(
    _ matchID: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            return try await withCheckedThrowingContinuation { continuation in
                match.rematch { rematch, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else if let rematch {
                        continuation.resume(returning: rematch)
                    } else {
                        continuation.resume(throwing: GKBridgeError.unknown("turn-based rematch returned nil"))
                    }
                }
            }
        },
        onSuccess: { match in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(try gkTurnBasedMatchPayload(from: match))) ?? "{}")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_turn_based_accept_invite_json")
public func gk_turn_based_accept_invite_json(
    _ matchID: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            return try await withCheckedThrowingContinuation { continuation in
                match.acceptInvite { acceptedMatch, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else if let acceptedMatch {
                        continuation.resume(returning: acceptedMatch)
                    } else {
                        continuation.resume(throwing: GKBridgeError.unknown("acceptInvite returned nil"))
                    }
                }
            }
        },
        onSuccess: { match in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(try gkTurnBasedMatchPayload(from: match))) ?? "{}")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_turn_based_decline_invite")
public func gk_turn_based_decline_invite(
    _ matchID: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            return try await withCheckedThrowingContinuation { continuation in
                match.declineInvite { error in
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

@_cdecl("gk_turn_based_remove")
public func gk_turn_based_remove(
    _ matchID: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            return try await withCheckedThrowingContinuation { continuation in
                match.remove { error in
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

@_cdecl("gk_turn_based_load_match_data_json")
public func gk_turn_based_load_match_data_json(
    _ matchID: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            return try await withCheckedThrowingContinuation { continuation in
                match.loadMatchData { data, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: data ?? Data())
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

@_cdecl("gk_turn_based_save_current_turn")
public func gk_turn_based_save_current_turn(
    _ matchID: UnsafePointer<CChar>?,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            let matchData = data.map { Data(bytes: $0, count: len) } ?? Data()
            return try await withCheckedThrowingContinuation { continuation in
                match.saveCurrentTurn(withMatch: matchData) { error in
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

@_cdecl("gk_turn_based_end_turn")
public func gk_turn_based_end_turn(
    _ matchID: UnsafePointer<CChar>?,
    _ nextIndicesJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            let indices = try gkDecodeJSON(nextIndicesJSON, as: [Int].self)
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            let participants = try gkTurnBasedParticipants(from: indices, in: match)
            let matchData = data.map { Data(bytes: $0, count: len) } ?? Data()
            return try await withCheckedThrowingContinuation { continuation in
                match.endTurn(withNextParticipants: participants, turnTimeout: timeoutSeconds, match: matchData) { error in
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

@_cdecl("gk_turn_based_participant_quit_in_turn")
public func gk_turn_based_participant_quit_in_turn(
    _ matchID: UnsafePointer<CChar>?,
    _ outcome: Int32,
    _ nextIndicesJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            let indices = try gkDecodeJSON(nextIndicesJSON, as: [Int].self)
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            let participants = try gkTurnBasedParticipants(from: indices, in: match)
            let matchData = data.map { Data(bytes: $0, count: len) } ?? Data()
            let outcomeValue = GKTurnBasedMatch.Outcome(rawValue: Int(outcome)) ?? .none
            return try await withCheckedThrowingContinuation { continuation in
                match.participantQuitInTurn(with: outcomeValue, nextParticipants: participants, turnTimeout: timeoutSeconds, match: matchData) { error in
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

@_cdecl("gk_turn_based_participant_quit_out_of_turn")
public func gk_turn_based_participant_quit_out_of_turn(
    _ matchID: UnsafePointer<CChar>?,
    _ outcome: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            let outcomeValue = GKTurnBasedMatch.Outcome(rawValue: Int(outcome)) ?? .none
            return try await withCheckedThrowingContinuation { continuation in
                match.participantQuitOutOfTurn(with: outcomeValue) { error in
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

@_cdecl("gk_turn_based_end_match_in_turn")
public func gk_turn_based_end_match_in_turn(
    _ matchID: UnsafePointer<CChar>?,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ scoresJSON: UnsafePointer<CChar>?,
    _ achievementsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            let matchData = data.map { Data(bytes: $0, count: len) } ?? Data()
            let scores = if let scoresJSON {
                try await gkBuildLeaderboardScores(
                    from: try gkDecodeJSON(scoresJSON, as: [GKScoreInputPayload].self),
                    preferredPlayers: match.participants.compactMap(\.player)
                )
            } else {
                [GKLeaderboardScore]()
            }
            let achievements = if let achievementsJSON {
                try await gkBuildAchievements(
                    from: try gkDecodeJSON(achievementsJSON, as: [GKAchievementInputPayload].self),
                    match: match
                )
            } else {
                [GKAchievement]()
            }
            return try await withCheckedThrowingContinuation { continuation in
                match.endMatchInTurn(withMatch: matchData, leaderboardScores: scores, achievements: achievements) { error in
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

@_cdecl("gk_turn_based_save_merged_match_data")
public func gk_turn_based_save_merged_match_data(
    _ matchID: UnsafePointer<CChar>?,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ resolvedIndicesJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID else {
                throw GKBridgeError.unknown("missing match identifier")
            }
            let indices = try gkDecodeJSON(resolvedIndicesJSON, as: [Int].self)
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            let exchanges = try indices.map { try gkTurnBasedExchange(at: $0, in: match) }
            let matchData = data.map { Data(bytes: $0, count: len) } ?? Data()
            return try await withCheckedThrowingContinuation { continuation in
                match.saveMergedMatch(matchData, withResolvedExchanges: exchanges) { error in
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

@_cdecl("gk_turn_based_send_exchange_json")
public func gk_turn_based_send_exchange_json(
    _ matchID: UnsafePointer<CChar>?,
    _ participantIndicesJSON: UnsafePointer<CChar>?,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ key: UnsafePointer<CChar>?,
    _ argsJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID, let key else {
                throw GKBridgeError.unknown("missing exchange parameters")
            }
            let indices = try gkDecodeJSON(participantIndicesJSON, as: [Int].self)
            let arguments = try gkDecodeJSON(argsJSON, as: [String].self)
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            let participants = try gkTurnBasedParticipants(from: indices, in: match)
            let exchangeData = data.map { Data(bytes: $0, count: len) } ?? Data()
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<GKTurnBasedExchange, Error>) in
                match.sendExchange(to: participants, data: exchangeData, localizableMessageKey: String(cString: key), arguments: arguments, timeout: timeoutSeconds) { exchange, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else if let exchange {
                        continuation.resume(returning: exchange)
                    } else {
                        continuation.resume(throwing: GKBridgeError.unknown("sendExchange returned nil"))
                    }
                }
            }
        },
        onSuccess: { (exchange: GKTurnBasedExchange) in
            let payloadParticipants = exchange.recipients + [exchange.sender]
            outJSON?.pointee = gkCString((try? gkEncodeJSON(gkTurnBasedExchangePayload(from: exchange, index: 0, participants: payloadParticipants))) ?? "{}")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_turn_based_cancel_exchange")
public func gk_turn_based_cancel_exchange(
    _ matchID: UnsafePointer<CChar>?,
    _ exchangeIndex: Int,
    _ key: UnsafePointer<CChar>?,
    _ argsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID, let key else {
                throw GKBridgeError.unknown("missing exchange parameters")
            }
            let arguments = try gkDecodeJSON(argsJSON, as: [String].self)
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            let exchange = try gkTurnBasedExchange(at: exchangeIndex, in: match)
            return try await withCheckedThrowingContinuation { continuation in
                exchange.cancel(withLocalizableMessageKey: String(cString: key), arguments: arguments) { error in
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

@_cdecl("gk_turn_based_reply_exchange")
public func gk_turn_based_reply_exchange(
    _ matchID: UnsafePointer<CChar>?,
    _ exchangeIndex: Int,
    _ key: UnsafePointer<CChar>?,
    _ argsJSON: UnsafePointer<CChar>?,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID, let key else {
                throw GKBridgeError.unknown("missing exchange parameters")
            }
            let arguments = try gkDecodeJSON(argsJSON, as: [String].self)
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            let exchange = try gkTurnBasedExchange(at: exchangeIndex, in: match)
            let exchangeData = data.map { Data(bytes: $0, count: len) } ?? Data()
            return try await withCheckedThrowingContinuation { continuation in
                exchange.reply(withLocalizableMessageKey: String(cString: key), arguments: arguments, data: exchangeData) { error in
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

@_cdecl("gk_turn_based_send_reminder")
public func gk_turn_based_send_reminder(
    _ matchID: UnsafePointer<CChar>?,
    _ participantIndicesJSON: UnsafePointer<CChar>?,
    _ key: UnsafePointer<CChar>?,
    _ argsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let matchID, let key else {
                throw GKBridgeError.unknown("missing reminder parameters")
            }
            let indices = try gkDecodeJSON(participantIndicesJSON, as: [Int].self)
            let arguments = try gkDecodeJSON(argsJSON, as: [String].self)
            let match = try await gkLoadTurnBasedMatch(with: String(cString: matchID))
            let participants = try gkTurnBasedParticipants(from: indices, in: match)
            return try await withCheckedThrowingContinuation { continuation in
                match.sendReminder(to: participants, localizableMessageKey: String(cString: key), arguments: arguments) { error in
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
