import Foundation
import GameKit

struct GKLocalPlayerPayload: Codable {
    let isAuthenticated: Bool
    let isUnderage: Bool
    let isMultiplayerGamingRestricted: Bool
    let isPersonalizedCommunicationRestricted: Bool
    let isPresentingFriendRequestViewController: Bool
    let player: GKPlayerPayload
}

struct GKIdentityVerificationSignaturePayload: Codable {
    let publicKeyURL: String
    let signatureBase64: String
    let saltBase64: String
    let timestamp: UInt64
}

struct GKAuthEventPayload: Encodable {
    let hasViewController: Bool
    let error: GKFrameworkErrorPayload?
}

public typealias GKAuthCallbackFn = @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> Void

private var gkAuthCallback: GKAuthCallbackFn?
private var gkAuthRefcon: UnsafeMutableRawPointer?

private func gkLocalPlayerPayload() -> GKLocalPlayerPayload {
    let player = GKLocalPlayer.local
    return GKLocalPlayerPayload(
        isAuthenticated: player.isAuthenticated,
        isUnderage: player.isUnderage,
        isMultiplayerGamingRestricted: player.isMultiplayerGamingRestricted,
        isPersonalizedCommunicationRestricted: player.isPersonalizedCommunicationRestricted,
        isPresentingFriendRequestViewController: player.isPresentingFriendRequestViewController,
        player: gkPlayerPayload(from: player)
    )
}

@_cdecl("gk_local_player_json")
public func gk_local_player_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        outJSON?.pointee = gkCString(try gkEncodeJSON(gkLocalPlayerPayload()))
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_authenticate_handler_set")
public func gk_authenticate_handler_set(
    _ callback: GKAuthCallbackFn?,
    _ refcon: UnsafeMutableRawPointer?
) {
    gkAuthCallback = callback
    gkAuthRefcon = refcon

    GKLocalPlayer.local.authenticateHandler = { viewController, error in
        guard let callback = gkAuthCallback else {
            return
        }

        let authError: GKFrameworkErrorPayload?
        if let error {
            let nsError = error as NSError
            authError = GKFrameworkErrorPayload(
                domain: nsError.domain,
                code: nsError.code,
                localizedDescription: nsError.localizedDescription
            )
        } else {
            authError = nil
        }

        let payload = GKAuthEventPayload(
            hasViewController: viewController != nil,
            error: authError
        )

        if let json = try? gkEncodeJSON(payload), let cString = gkCString(json) {
            defer { free(cString) }
            callback(gkAuthRefcon, UnsafePointer(cString))
        }
    }
}

@_cdecl("gk_authenticate_handler_clear")
public func gk_authenticate_handler_clear() {
    GKLocalPlayer.local.authenticateHandler = nil
    gkAuthCallback = nil
    gkAuthRefcon = nil
}

@_cdecl("gk_local_player_load_recent_players_json")
public func gk_local_player_load_recent_players_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKPlayer], Error>) in
                GKLocalPlayer.local.loadRecentPlayers { players, error in
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

@_cdecl("gk_local_player_load_challengable_friends_json")
public func gk_local_player_load_challengable_friends_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKPlayer], Error>) in
                GKLocalPlayer.local.loadChallengableFriends { players, error in
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

@_cdecl("gk_local_player_fetch_identity_verification_signature_json")
public func gk_local_player_fetch_identity_verification_signature_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { continuation in
                GKLocalPlayer.local.fetchItems(forIdentityVerificationSignature: { publicKeyURL, signature, salt, timestamp, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else if let publicKeyURL, let signature, let salt {
                        continuation.resume(returning: GKIdentityVerificationSignaturePayload(
                            publicKeyURL: publicKeyURL.absoluteString,
                            signatureBase64: signature.base64EncodedString(),
                            saltBase64: salt.base64EncodedString(),
                            timestamp: timestamp
                        ))
                    } else {
                        continuation.resume(throwing: GKBridgeError.unknown("identity verification signature was incomplete"))
                    }
                })
            }
        },
        onSuccess: { (payload: GKIdentityVerificationSignaturePayload) in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(payload)) ?? "{}")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_local_player_load_friends_authorization_status")
public func gk_local_player_load_friends_authorization_status(
    _ outStatus: UnsafeMutablePointer<Int32>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<GKFriendsAuthorizationStatus, Error>) in
                GKLocalPlayer.local.loadFriendsAuthorizationStatus { status, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: status)
                    }
                }
            }
        },
        onSuccess: { (status: GKFriendsAuthorizationStatus) in
            outStatus?.pointee = Int32(status.rawValue)
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_local_player_load_friends_json")
public func gk_local_player_load_friends_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKPlayer], Error>) in
                GKLocalPlayer.local.loadFriends { friends, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: friends ?? [])
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

@_cdecl("gk_local_player_load_friends_by_identifiers_json")
public func gk_local_player_load_friends_by_identifiers_json(
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let identifiers = try gkDecodeJSON(identifiersJSON, as: [String].self)
            return try await gkLoadPlayers(identifiedBy: identifiers)
        },
        onSuccess: { (players: [GKPlayer]) in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(players.map(gkPlayerPayload(from:)))) ?? "[]")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_local_player_present_friend_request")
public func gk_local_player_present_friend_request(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        if #available(macOS 12.0, *) {
            try GKLocalPlayer.local.presentFriendRequestCreator(from: nil)
            return GK_OK
        }
        throw GKBridgeError.unavailable("presenting a friend request requires macOS 12.0 or newer")
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}
