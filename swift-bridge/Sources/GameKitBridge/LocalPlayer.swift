import AppKit
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

final class GKAuthHub {
    static let shared = GKAuthHub()

    private let lock = NSLock()
    private let installLock = NSRecursiveLock()
    private var subscribers: [UInt64: (NSViewController?, Error?) -> Bool] = [:]
    private var nextToken: UInt64 = 1
    private var pendingViewController: NSViewController?

    func subscribe(_ handler: @escaping (NSViewController?, Error?) -> Bool) -> UInt64 {
        lock.lock()
        let token = nextToken
        nextToken += 1
        subscribers[token] = handler
        lock.unlock()

        installLock.lock()
        GKLocalPlayer.local.authenticateHandler = { viewController, error in
            GKAuthHub.shared.dispatch(viewController, error)
        }
        installLock.unlock()
        return token
    }

    func unsubscribe(_ token: UInt64) {
        lock.lock()
        subscribers.removeValue(forKey: token)
        lock.unlock()

        installLock.lock()
        lock.lock()
        let empty = subscribers.isEmpty
        lock.unlock()
        if empty {
            GKLocalPlayer.local.authenticateHandler = nil
        }
        installLock.unlock()
    }

    func signInViewController() -> NSViewController? {
        lock.lock()
        defer { lock.unlock() }
        return pendingViewController
    }

    private func dispatch(_ viewController: NSViewController?, _ error: Error?) {
        lock.lock()
        pendingViewController = viewController
        let current = subscribers
        lock.unlock()

        var finished: [UInt64] = []
        for (token, handler) in current where !handler(viewController, error) {
            finished.append(token)
        }
        guard !finished.isEmpty else { return }
        lock.lock()
        for token in finished {
            subscribers.removeValue(forKey: token)
        }
        lock.unlock()
    }
}

private final class GKAuthObserverSink {
    let callback: GKAuthCallbackFn
    let refcon: UnsafeMutableRawPointer?
    let releaseContext: GKContextCallback?

    init(
        callback: @escaping GKAuthCallbackFn,
        refcon: UnsafeMutableRawPointer?,
        retainContext: GKContextCallback?,
        releaseContext: GKContextCallback?
    ) {
        self.callback = callback
        self.refcon = refcon
        self.releaseContext = releaseContext
        retainContext?(refcon)
    }

    deinit {
        releaseContext?(refcon)
    }

    func deliver(_ viewController: NSViewController?, _ error: Error?) {
        let authError = error.map { error -> GKFrameworkErrorPayload in
            let nsError = error as NSError
            return GKFrameworkErrorPayload(
                domain: nsError.domain,
                code: nsError.code,
                localizedDescription: nsError.localizedDescription
            )
        }
        let payload = GKAuthEventPayload(hasViewController: viewController != nil, error: authError)
        if let json = try? gkEncodeJSON(payload), let cString = gkCString(json) {
            defer { free(cString) }
            callback(refcon, UnsafePointer(cString))
        }
    }
}

func gkLocalPlayerPayload() -> GKLocalPlayerPayload {
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
    _ refcon: UnsafeMutableRawPointer?,
    _ retainContext: GKContextCallback?,
    _ releaseContext: GKContextCallback?
) -> UInt64 {
    guard let callback else { return 0 }
    let sink = GKAuthObserverSink(
        callback: callback,
        refcon: refcon,
        retainContext: retainContext,
        releaseContext: releaseContext
    )
    return GKAuthHub.shared.subscribe { viewController, error in
        sink.deliver(viewController, error)
        return true
    }
}

@_cdecl("gk_authenticate_handler_clear")
public func gk_authenticate_handler_clear(_ token: UInt64) {
    guard token != 0 else { return }
    GKAuthHub.shared.unsubscribe(token)
}

@_cdecl("gk_local_player_sign_in_view_controller")
public func gk_local_player_sign_in_view_controller(
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        try gkRequireMainThread("reading the Game Center sign-in view controller")
        outPtr?.pointee = GKAuthHub.shared.signInViewController().map { gk_retain($0) }
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_sign_in_view_controller_present")
public func gk_sign_in_view_controller_present(
    _ ptr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        try gkRequireMainThread("presenting the Game Center sign-in view controller")
        guard let ptr else {
            throw GKBridgeError.unknown("null sign-in view controller pointer")
        }
        let viewController = gk_borrow(ptr, as: NSViewController.self)
        if let presentable = viewController as? (NSViewController & GKViewController) {
            let dialog = try gkDialogControllerForPresentation()
            guard dialog.present(presentable) else {
                throw GKBridgeError.unknown("GKDialogController refused to present the sign-in view controller")
            }
            return GK_OK
        }
        let window = NSApplication.shared.mainWindow ?? NSApplication.shared.keyWindow
        guard let host = window?.contentViewController else {
            throw GKBridgeError.notFound(
                "no window content view controller is available to present the sign-in view controller"
            )
        }
        host.presentAsSheet(viewController)
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_view_controller_release")
public func gk_view_controller_release(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    gk_release(ptr)
}

@_cdecl("gk_local_player_load_recent_players_json")
public func gk_local_player_load_recent_players_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            try Task.checkCancellation()
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKPlayer], Error>) in
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
            try Task.checkCancellation()
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKPlayer], Error>) in
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
            try Task.checkCancellation()
            return try await withCheckedThrowingContinuation { continuation in
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
            try Task.checkCancellation()
            try gkRequireBundleIdentifier("loading the friends authorization status")
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<GKFriendsAuthorizationStatus, Error>) in
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
            try Task.checkCancellation()
            try gkRequireBundleIdentifier("loading friends")
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKPlayer], Error>) in
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
    let identifiersJSON = gkCopyString(identifiersJSON)
    return gkBlockOnAsync(
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
