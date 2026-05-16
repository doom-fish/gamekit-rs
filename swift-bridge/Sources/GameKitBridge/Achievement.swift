import Foundation
import GameKit

struct GKAchievementInputPayload: Codable {
    let identifier: String
    let percentComplete: Double
}

struct GKAchievementResultPayload: Codable {
    let identifier: String
    let percentComplete: Double
    let isCompleted: Bool
    let lastReportedDate: String
}

@_cdecl("gk_achievement_report_json")
public func gk_achievement_report_json(
    _ achievementsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    return gkBlockOnAsync(
        work: {
            let inputs = try gkDecodeJSON(achievementsJSON, as: [GKAchievementInputPayload].self)
            let achievements = inputs.map { input -> GKAchievement in
                let achievement = GKAchievement(identifier: input.identifier)
                achievement.percentComplete = input.percentComplete
                return achievement
            }
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                GKAchievement.report(achievements) { error in
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
