import Foundation
import GameKit

struct GKScoreInputPayload: Codable {
    let leaderboardID: String
    let value: Int64
    let context: UInt64
    let playerGameID: String?
}

func gkBuildLeaderboardScores(
    from inputs: [GKScoreInputPayload],
    preferredPlayers: [GKPlayer] = []
) async throws -> [GKLeaderboardScore] {
    let playersByID = try await gkResolvePlayers(
        byGameIDs: inputs.compactMap(\.playerGameID),
        preferred: preferredPlayers
    )

    return inputs.map { input in
        let score = GKLeaderboardScore()
        score.leaderboardID = input.leaderboardID
        score.value = Int(input.value)
        score.context = Int(input.context)
        if let playerGameID = input.playerGameID, let player = playersByID[playerGameID] {
            score.player = player
        } else {
            score.player = GKLocalPlayer.local
        }
        return score
    }
}

@_cdecl("gk_score_report_json")
public func gk_score_report_json(
    _ scoresJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let inputs = try gkDecodeJSON(scoresJSON, as: [GKScoreInputPayload].self)
            let playersByID = try await gkResolvePlayers(
                byGameIDs: inputs.compactMap(\.playerGameID)
            )
            let scores: [GKScore] = inputs.map { input in
                let score: GKScore
                if let playerGameID = input.playerGameID, let player = playersByID[playerGameID] {
                    score = GKScore(leaderboardIdentifier: input.leaderboardID, player: player)
                } else {
                    score = GKScore(leaderboardIdentifier: input.leaderboardID)
                }
                score.value = input.value
                score.context = input.context
                return score
            }
            return try await withCheckedThrowingContinuation { continuation in
                GKScore.report(scores) { error in
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
