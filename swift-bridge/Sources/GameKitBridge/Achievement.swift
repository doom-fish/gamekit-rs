import Foundation
import GameKit

struct GKAchievementInputPayload: Codable {
    let identifier: String
    let percentComplete: Double
    let showsCompletionBanner: Bool
    let playerGameID: String?
}

struct GKAchievementPayload: Codable {
    let identifier: String
    let percentComplete: Double
    let isCompleted: Bool
    let lastReportedDate: String?
    let showsCompletionBanner: Bool
    let player: GKPlayerPayload?
}

struct GKAchievementDescriptionPayload: Codable {
    let identifier: String?
    let groupIdentifier: String?
    let title: String?
    let achievedDescription: String?
    let unachievedDescription: String?
    let maximumPoints: Int
    let isHidden: Bool
    let isReplayable: Bool
}

func gkAchievementPayload(from achievement: GKAchievement) -> GKAchievementPayload {
    GKAchievementPayload(
        identifier: achievement.identifier ?? "",
        percentComplete: achievement.percentComplete,
        isCompleted: achievement.isCompleted,
        lastReportedDate: gkDateString(achievement.lastReportedDate),
        showsCompletionBanner: achievement.showsCompletionBanner,
        player: gkPlayerPayload(from: achievement.player)
    )
}

func gkAchievementDescriptionPayload(from description: GKAchievementDescription) -> GKAchievementDescriptionPayload {
    GKAchievementDescriptionPayload(
        identifier: description.identifier,
        groupIdentifier: description.groupIdentifier,
        title: description.title,
        achievedDescription: description.achievedDescription,
        unachievedDescription: description.unachievedDescription,
        maximumPoints: description.maximumPoints,
        isHidden: description.isHidden,
        isReplayable: description.isReplayable
    )
}

func gkBuildAchievements(
    from inputs: [GKAchievementInputPayload],
    preferredPlayers: [GKPlayer] = []
) async throws -> [GKAchievement] {
    let playerIDs = inputs.compactMap(\.playerGameID)
    let playersByID = try await gkResolvePlayers(byGameIDs: playerIDs, preferred: preferredPlayers)

    return inputs.map { input in
        let achievement: GKAchievement
        if let playerGameID = input.playerGameID, let player = playersByID[playerGameID] {
            achievement = GKAchievement(identifier: input.identifier, player: player)
        } else {
            achievement = GKAchievement(identifier: input.identifier)
        }
        achievement.percentComplete = input.percentComplete
        achievement.showsCompletionBanner = input.showsCompletionBanner
        return achievement
    }
}

@_cdecl("gk_achievement_load_json")
public func gk_achievement_load_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKAchievement], Error>) in
                GKAchievement.loadAchievements { achievements, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: achievements ?? [])
                    }
                }
            }
        },
        onSuccess: { (achievements: [GKAchievement]) in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(achievements.map(gkAchievementPayload(from:)))) ?? "[]")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_achievement_descriptions_json")
public func gk_achievement_descriptions_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKAchievementDescription], Error>) in
                GKAchievementDescription.loadAchievementDescriptions { descriptions, error in
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

@_cdecl("gk_achievement_report_json")
public func gk_achievement_report_json(
    _ achievementsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let inputs = try gkDecodeJSON(achievementsJSON, as: [GKAchievementInputPayload].self)
            let achievements = try await gkBuildAchievements(from: inputs)
            return try await withCheckedThrowingContinuation { continuation in
                GKAchievement.report(achievements) { error in
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

@_cdecl("gk_achievement_reset")
public func gk_achievement_reset(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { continuation in
                GKAchievement.resetAchievements { error in
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
