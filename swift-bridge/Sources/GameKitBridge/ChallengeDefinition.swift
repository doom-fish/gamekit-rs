import AppKit
import Foundation
import GameKit

#if GAMEKIT_HAS_MACOS26_SDK
struct GKChallengeDurationOptionPayload: Codable {
    let year: Int?
    let month: Int?
    let weekOfYear: Int?
    let day: Int?
    let hour: Int?
    let minute: Int?
    let second: Int?
}

struct GKChallengeDefinitionPayload: Codable {
    let identifier: String
    let groupIdentifier: String?
    let title: String
    let details: String?
    let durationOptions: [GKChallengeDurationOptionPayload]
    let isRepeatable: Bool
    let leaderboardID: String?
    let releaseState: String
}

@available(macOS 26.0, *)
private func gkChallengeReleaseState(_ state: GKReleaseState) -> String {
    switch state {
    case .released:
        return "released"
    case .prereleased:
        return "prereleased"
    default:
        return "unknown"
    }
}

@available(macOS 26.0, *)
private func gkChallengeDefinitionPayload(from definition: GKChallengeDefinition) -> GKChallengeDefinitionPayload {
    GKChallengeDefinitionPayload(
        identifier: definition.identifier,
        groupIdentifier: definition.groupIdentifier,
        title: definition.title,
        details: definition.details,
        durationOptions: definition.durationOptions.map {
            GKChallengeDurationOptionPayload(
                year: $0.year,
                month: $0.month,
                weekOfYear: $0.weekOfYear,
                day: $0.day,
                hour: $0.hour,
                minute: $0.minute,
                second: $0.second
            )
        },
        isRepeatable: definition.isRepeatable,
        leaderboardID: definition.leaderboard?.baseLeaderboardID,
        releaseState: gkChallengeReleaseState(definition.releaseState)
    )
}

@_cdecl("gk_challenge_definitions_json")
public func gk_challenge_definitions_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKChallengeDefinition], Error>) in
                    GKChallengeDefinition.loadChallengeDefinitions { definitions, error in
                        if let error {
                            continuation.resume(throwing: error)
                        } else {
                            continuation.resume(returning: definitions ?? [])
                        }
                    }
                }
            },
            onSuccess: { (definitions: [GKChallengeDefinition]) in
                outJSON?.pointee = gkCString((try? gkEncodeJSON(definitions.map(gkChallengeDefinitionPayload(from:)))) ?? "[]")
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKChallengeDefinition requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_challenge_definition_has_active")
public func gk_challenge_definition_has_active(
    _ identifier: UnsafePointer<CChar>?,
    _ outActive: UnsafeMutablePointer<Bool>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                guard let identifier else {
                    throw GKBridgeError.unknown("missing challenge definition identifier")
                }
                let definitions = try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKChallengeDefinition], Error>) in
                    GKChallengeDefinition.loadChallengeDefinitions { definitions, error in
                        if let error {
                            continuation.resume(throwing: error)
                        } else {
                            continuation.resume(returning: definitions ?? [])
                        }
                    }
                }
                guard let definition = definitions.first(where: { $0.identifier == String(cString: identifier) }) else {
                    throw GKBridgeError.notFound("challenge definition not found")
                }
                return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Bool, Error>) in
                    definition.hasActiveChallenges { hasActiveChallenges, error in
                        if let error {
                            continuation.resume(throwing: error)
                        } else {
                            continuation.resume(returning: hasActiveChallenges)
                        }
                    }
                }
            },
            onSuccess: { (hasActiveChallenges: Bool) in
                outActive?.pointee = hasActiveChallenges
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKChallengeDefinition requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_challenge_definition_load_image_json")
public func gk_challenge_definition_load_image_json(
    _ identifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 26.0, *) {
        return gkBlockOnAsync(
            work: {
                guard let identifier else {
                    throw GKBridgeError.unknown("missing challenge definition identifier")
                }
                let definitions = try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKChallengeDefinition], Error>) in
                    GKChallengeDefinition.loadChallengeDefinitions { definitions, error in
                        if let error {
                            continuation.resume(throwing: error)
                        } else {
                            continuation.resume(returning: definitions ?? [])
                        }
                    }
                }
                guard let definition = definitions.first(where: { $0.identifier == String(cString: identifier) }) else {
                    throw GKBridgeError.notFound("challenge definition not found")
                }
                let image = try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<NSImage, Error>) in
                    definition.loadImage { image, error in
                        if let error {
                            continuation.resume(throwing: error)
                        } else if let image {
                            continuation.resume(returning: image)
                        } else {
                            continuation.resume(throwing: GKBridgeError.unknown("challenge definition image load returned nil"))
                        }
                    }
                }
                guard let data = image.tiffRepresentation else {
                    throw GKBridgeError.unknown("challenge definition image did not expose TIFF data")
                }
                return data
            },
            onSuccess: { data in
                outJSON?.pointee = gkCString((try? gkBinaryPayloadJSON(data)) ?? "{}")
            },
            onError: { error in
                gkPopulateError(outError, with: error)
            }
        )
    }
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKChallengeDefinition requires macOS 26.0 or newer"))
    return GK_UNAVAILABLE
}
#else
@_cdecl("gk_challenge_definitions_json")
public func gk_challenge_definitions_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKChallengeDefinition requires the macOS 26 SDK"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_challenge_definition_has_active")
public func gk_challenge_definition_has_active(
    _ identifier: UnsafePointer<CChar>?,
    _ outActive: UnsafeMutablePointer<Bool>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKChallengeDefinition requires the macOS 26 SDK"))
    return GK_UNAVAILABLE
}

@_cdecl("gk_challenge_definition_load_image_json")
public func gk_challenge_definition_load_image_json(
    _ identifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkPopulateError(outError, with: GKBridgeError.unavailable("GKChallengeDefinition requires the macOS 26 SDK"))
    return GK_UNAVAILABLE
}
#endif
